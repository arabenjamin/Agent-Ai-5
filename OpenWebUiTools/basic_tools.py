import os, os.path, asyncio
import requests
import datetime
import logging
import json
from datetime import datetime as dt
from typing import Any, Callable, List

from google.auth.transport.requests import Request
from google.auth.external_account_authorized_user import Credentials as ExternalCredentials
from google.oauth2.credentials import Credentials as OauthCredentials
from google_auth_oauthlib.flow import Flow
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError

from pydantic import BaseModel, Field




numeric_log_level = getattr(logging, "DEBUG", None)
if not isinstance(numeric_log_level, int):
    raise ValueError('Invalid log level: %s' % "DEBUG")

# Set up logging
logger = logging.getLogger(__name__)
logging.basicConfig(

    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    filename='dev_vendor_parser.log', 
    encoding='utf-8', 
    level=numeric_log_level
)


async def authenticate_with_google(
    scopes: List[str], credentials_json_string: str, token_file_path: str, __event_emitter__: Callable[[dict], Any] = None, __event_call__: Callable[[dict], Any]=None
) -> OauthCredentials | ExternalCredentials | str:
    """Aunthenticate with google return the credentials"""

    print("\n\n*** Authenticating with Google Calendar API... ***\n\n")
    event_emitter = EventEmitter(__event_emitter__)
    await event_emitter.progress_update("Authenticating with Google Calendar API...")
    creds = None
    # The file token.json stores the user's access and refresh tokens, and is
    # created automatically when the authorization flow completes for the first
    # time.

    if os.path.exists(token_file_path):
        try:
            creds = OauthCredentials.from_authorized_user_file(token_file_path, scopes)
        except Exception as e:
            await event_emitter.error_update(
                f"Error loading credentials from {token_file_path}: {e}"
            )
            print(f"\n\n*** ERROR loading file ... ***\n{e}\n")
            creds = None
            print(f"\n\n*** Moving Forward... ***\n{e}\n")
    

    

    print("\n\n*** ONWARD!!! ***\n{creds}\n")
    await event_emitter.progress_update("Moving forward with authentication...")

    # If there are no (valid) credentials available, let the user log in.
    if not creds or not creds.valid:
        print("\n\n*** Our creds are not valid... ***\n\n")

        await event_emitter.progress_update("Credentials invalid Moving forward with authentication...")
        # If the credentials are expired, try to refresh them
        if creds and creds.expired and creds.refresh_token:

            try:
                print("\n\n*** Attempting to Refresh credentials... ***\n\n")
                await event_emitter.progress_update("Refreshing credentials...")
                creds.refresh(Request())
            except Exception as e:
                # Log error for debugging purposes
                print(
                    f"\n\n*** Error refreshing token from {token_file_path}: {e}. Need to re-authenticate.***\n\n"
                )
                await event_emitter.error_update(
                    f"Error refreshing token from {token_file_path}: {e}. Need to re-authenticate."
                )
                # Remove potentially corrupt token file if refresh fails
                if os.path.exists(token_file_path):
                    os.remove(token_file_path)
                creds = None
        else:

            try:
                print("\n\n*** Attempting to authenticate with Google via login flow ... ***\n\n")
                await event_emitter.progress_update("Authenticating with Google via login flow...")
                # If there are no (valid) credentials available, let the user log in.
                # Use the InstalledAppFlow to authenticate
                print("\n\n*** Creating InstalledAppFlow ... ***\n\n")
                await event_emitter.progress_update("Creating InstalledAppFlow...")
                client_config = json.loads(credentials_json_string)
                # Attempt to run the local server flow
                try:
                    flow = InstalledAppFlow.from_client_config(client_config, scopes)
                    flow.redirect_uri = "http://localhost:8080/oauth/google/callback/"
                    await event_emitter.progress_update("Flow Created...")
                except Exception as e:
                    print(f"\n\n*** FLOW ERRROR!!!!\n{e}\n")
                    await event_emitter.error_update(f"\n\n*** ERRROR!!!!\n{e}\n")

                print("\n\n*** Running local server to finish flow ***\n\n")
                await event_emitter.progress_update("Running local server to finish flow")
                # Set prompt=consent to ensure refresh token is issued the first time

                print(f"*** Flow Redirect URI: {flow.redirect_uri}***")
                auth_url = flow.authorization_url()
                print(f"*** Auth URL: {auth_url[0]}***")

                flow.run_local_server(port=0, open_browser=False)

                await event_emitter.message_update(auth_url[0])
                print(f"*** Flow Authorization URL: {auth_url}***")
                #user_confirmation = await __event_call__(
                #    {
                #        "type": "confirmation",
                #        "data": {
                #            "title": "Are you sure?",
                #            "message": auth_url[0],
                #        },
                #    }
                #)
                #print(f"\n\n***User Confirmation: ***\n{user_confirmation}\n")
                creds = flow.credentials

                print("\n\n*** Successfully authenticated with Google ***\n{creds}\n")
                await event_emitter.success_update(
                    "Successfully authenticated with Google Calendar API."
                )
            except Exception as e:
                print(f"\n\n*** Google Authentication flow failed ***\n{e}\n")
                await event_emitter.error_update(f"\n\n*** Google Authentication flow failed ***\n{e}\n")
                raise Exception(f"\n\n*** FAILED TO AUTHENTICATE WITH GOOGLE ***\n{e}\n")

        # Save the credentials for the next run
        if creds and creds.valid:
            try:
                with open(token_file_path, "w") as token:
                    token.write(creds.to_json())
            except Exception as e:
                await event_emitter.error_update(
                    f"Could not save token file {token_file_path}: {e}"
                )
                print(f"\n\n*** Look out ! Could not save token file !! ***\n{e}\n")
                raise Exception(f"\n\n*** Look out ! Could not save token file !! ***\n{e}\n")
    # Save the credentials for the next run
    if creds and creds.valid:
        await event_emitter.success_update(
            f"Credentials are valid and saved to {token_file_path}."
        )
        return creds
    print(f"\n\n*** Authentication Failed ***\n{creds}")

    raise Exception(f"***Authentication Failure: Failed to Authenticate with Google \n{creds}***")

def get_geocoords(api_key: str, country: str, zipcode: str) -> dict:
    print("Getting Geo Coordintes for weather look up")
    url = f"http://api.openweathermap.org/geo/1.0/zip?zip={zipcode},{country}&appid={api_key}"
    resp = requests.get(url)
    print(f"Coords Response: {resp.status_code}")
    if resp.status_code == 200:
        return resp.json()
    return {}

class Equation(BaseModel):
    equation: str = Field(..., description="The mathematical equation to calculate.")

class City(BaseModel):
    city: str = Field(
        "Bothell, Wa", description="Get the current weather for a given city."
    )

class EventEmitter:
    def __init__(self, event_emitter: Callable[[dict], Any] = None, call_emitter: Callable[[dict], Any] = None):
        
        self.event_emitter = event_emitter
        print("\n\n*** Emitter initiated  ***\n\n")

    async def progress_update(self, description: str):
        """Emit a progress update."""
        print(f"\n\n*** Emitting progress update: {description}  ***\n\n")
        await self.emit(description)

    async def error_update(self, description: str):
        """Emit an error update and mark as done."""
        print(f"\n\n*** Emitting error!!: {description}  ***\n\n")
        await self.emit(description, "error", True)

    async def message_update(self, content: str):
        """Emit a message update."""
        print(f"\n\n*** Emitting message: {content}  ***\n\n")
        event_data = {
            "type": "chat:message:delta",
            "data": {
                "content": content,
            }
        }
        if self.event_emitter:
            await self.event_emitter(event_data)


    async def success_update(self, description: str, data: Any = None):
        """Emit a success update and mark as done."""
        print(f"\n\n*** Emitting Success: {description}  ***\n\n")
        event_data = {
            "type": "notification",
            "data": {
                "type": "info",
                "content": description,
            }
        }
        if data is not None:  # Allow empty dicts/lists as valid data
            event_data["data"]["details"] = data

        if self.event_emitter:
            await self.event_emitter(event_data)

    async def emit(
        self,
        description: str = "Unknown State",
        status: str = "in_progress",
        done: bool = False,
        type: str = "status",
    ):
        """Emit a generic status update."""
        print(f"***Emitting message: {description}***")
        if self.event_emitter:
            await self.event_emitter(
                {
                    "type": type,   # Type of event, e.g., "status", "notification", etc.
                    "data": {
                        "status": status,
                        "description": description,
                        "done": done,
                        "hidden": False,
                    },
                }
            )

class Tools:

    class Valves(BaseModel):
        api_key: str = Field(
            default=os.getenv("OPENWEATHERMAP_API_KEY", "NOTMYKEY"),
            description="Open Weather Map Api Key",
        )
        google_clientid: str = Field(
            default=os.getenv("GOOGLE_CLINETID", "NOTAKEY"),
            descriptiton="Google Api Credentials",
        )
        google_client_secret: str = Field(
            default=os.getenv("GOOGLE_CLIENT_SECRET", "NOTASECRET"),
            descriptiton="Google Api Credentials",
        )
        google_token: dict = Field(
            default=os.getenv("GOOGLE_TOKEN", "NOTASECRET"),
            descriptiton="Google Api Credentials",
        )
        

    def __init__(self, __event_emitter__: dict = {}, __event_call__: Callable[[dict], Any] = None):
        self.valves = self.Valves()
        self.token_file = "token.json"  # Use a specific token file
        
        if callable(__event_emitter__) and callable(__event_call__):
            self.emitter = EventEmitter(__event_emitter__, __event_call__)
        else:
            self.emitter = EventEmitter(lambda event: print(f"Event: {event}"), lambda event: asyncio.sleep(1))


        
    # Add your custom tools using pure Python code here, make sure to add type hints and descriptions

    async def example_tool(self, __event_emitter__: Callable[[dict], Any] = None, __event_call__: Callable[[dict], Any] = None) -> str:
        """
        An example tool that returns a dict.
        This is a placeholder for your custom tool.
        """
        print("\n\n*** Running Example Tool ***\n\n")
        # await self.emitter.progress_update("Running Example Tool...")
        emmiter_data = {
            "type": "status",  # We set the type here
            "data": {
                "description": "Running Example Tool!!",
                "hidden": False,
                "done": False,
            },
            # Note done is False here indicating we are still emitting statuses
        }
        await __event_emitter__(emmiter_data)

        print("Getting User input")
        resp = await __event_call__(
            {
                "type": "confirmation",  # Or "input" "confirmation", "execute"
                "data": {
                    "title": "Should I continue?",
                    "message": "Are we doing this?",
                    "placeholder": "Your password here",
                },
            }
        )

        result = {"type": "notification", "data": {"type": "info", "content": resp}}
        await __event_emitter__(result)

        return "Example tool Ran Successfully"

    async def get_user_name_and_email_and_id(self, __user__: dict = {}, __event_emitter__: Callable[[dict], Any] = None, __event_call__: Callable[[dict], Any] = None) -> str:
        """
        Get the user name, Email and ID from the user object.
        """
        event_emitter = EventEmitter(__event_emitter__)
        # Do not include a descrption for __user__ as it should not be shown in the tool's specification
        # The session user object will be passed as a parameter when the function is called

        print(f"\n\n*** USER!!{__user__} ***\n\n")
        await event_emitter.progress_update(
            f"Found :User {__user__['name']}, Email: {__user__['email']}"
        )
        # This is bad from and I cant stand it.
        result = ""

        if "name" in __user__:
            result += f"User: {__user__['name']}"
        if "id" in __user__:
            result += f" (ID: {__user__['id']})"
        if "email" in __user__:
            result += f" (Email: {__user__['email']})"

        if result == "":
            result = "User: Unknown"

        await event_emitter.success_update(
            f"Found User: {result}"
        )
        return result

    async def get_current_time(self, __event_emitter__: Callable[[dict], Any] = None, __event_call__: Callable[[dict], Any] = None) -> str:
        """
        Get the current time in a more human-readable format.
        """
        print("\n\n*** Getting Time ***\n\n")
        event_emitter = EventEmitter(__event_emitter__)
        await event_emitter.progress_update("Fetching current time...")
        now = dt.now()
        current_time = now.strftime("%I:%M:%S %p")  # Using 12-hour format with AM/PM
        current_date = now.strftime(
            "%A, %B %d, %Y"
        )  # Full weekday, month name, day, and year
        print(f"Current Date and Time = {current_date}, {current_time}")
        await event_emitter.success_update(
            f"Current Date and Time = {current_date}, {current_time}"
        )
        return f"Current Date and Time = {current_date}, {current_time}"

    async def get_current_weather(self, __event_emitter__: Callable[[dict], Any] = None, __event_call__: Callable[[dict], Any] = None) -> str:
        """Get the current weather for a given zipcode."""
        print("\n\n***GETTING CURRENT WEATHER***\n\n")
        
        event_emitter = EventEmitter(__event_emitter__)
        await event_emitter.progress_update("Fetching current weather...")
        """Get geocoords first"""

        try:

            zipcode = await __event_call__(
                {
                    "type": "input",
                    "data": {
                        "title": "Zipcode",
                        "message": "Please enter the Zipcode  (e.g., (98012))",
                        "placeholder": "Zipcode",
                    },
                }
            )
            coords = get_geocoords(self.valves.api_key, "US", zipcode)
            await event_emitter.progress_update(
                f"Found Geolocation coordinates LAT: {coords['lat']}, LONG: {coords['lon']} "
            )
            print(
                f"\n\n*** COORDS: LAT: {coords['lat']}, LONG: {coords['lon']} ***\n\n"
            )
        except Exception as e:
            print(
                f"***ERROR: There was an error retieving Coordinates for the location *** \n{e}\n"
            )
            return f"Error retrieving coordinates: {str(e)}"

        await event_emitter.progress_update(
            f"Fetching weather data for coordinates: LAT: {coords['lat']}, LONG: {coords['lon']}"
        )

        try:
            weather_url = f"https://api.openweathermap.org/data/2.5/weather?lat={coords['lat']}&lon={coords['lon']}&appid={self.valves.api_key}"
            response = requests.get(weather_url)
            response.raise_for_status()  # Raise HTTPError for bad responses (4xx and 5xx)
            data = response.json()
            print(f"\n\n***CURRENT WEATHER***\n{data}\n")
            await event_emitter.progress_update(
                f"Received weather data for {zipcode}."
            )
            if data.get("cod") != 200:
                await event_emitter.error_update(
                    f"Error fetching weather data: {data.get('message')}"
                )
                return f"Error fetching weather data: {data.get('message')}"

            weather_description = data["weather"][0]["description"]
            temperature = float(data["main"]["temp"]) - float(273.15)
            humidity = data["main"]["humidity"]
            wind_speed = data["wind"]["speed"]
            await event_emitter.success_update(
                f"Found Weather data for {zipcode}: {data['weather'][0]['description']}"
            ) 
            return f"Weather in {zipcode}: {weather_description}, Temp: {temperature}, Humidity: {humidity}, WindSpeed: {wind_speed}°C"
        except requests.RequestException as e:
            await event_emitter.error_update(
                f"Error fetching weather data: {str(e)}"
            )
            return f"Error fetching weather data: {str(e)}"

    def get_geolocation_and_public_ip(self) -> dict:
        url = "https://api.ipify.org?format=json"
        try:
            response = requests.get(url)
            if response.status_code == 200:
                json_data = response.json()
                print(
                    f"\n\n***DEBUG GEOLOCATION PUBLIC IP API JSON RESPONSE ***\n{json_data}\n"
                )
                public_ip = json_data["ip"]
                print(f"Public IP: {public_ip}")
                print("Retrieving geolocation data...")
                response = requests.get(
                    f"https://geo.ipify.org/json?ipAddress={public_ip}"
                )
                if response.status_code == 200:
                    json_data = response.json()
                    print(json_data)
                    return json_data
                else:
                    print(f"Failed to retrieve data: {response.status_code}")
            else:
                print(f"Failed to retrieve data: {response.status_code}")
        except requests.exceptions.RequestException as e:
            print(f"An error occurred: {e}")
        return {}

    async def get_my_calandar(self, __event_emitter__: Callable[[dict], Any] = None, __event_call__: Callable[[dict], Any] = None) -> dict:
        """
        Shows basic usage of the Google Calendar API.
        Prints the start and name of the next 10 events on the user's calendar.
        """
        event_emitter = EventEmitter(__event_emitter__)
        SCOPES = ["https://www.googleapis.com/auth/calendar.readonly"]

        await event_emitter.progress_update("Getting calendar events from Google Calendar API...")
        # TODO: Add your Google API credentials here or load them from a secure location
        secret_stuff = """ """
        

        try:
            creds = await authenticate_with_google(SCOPES, secret_stuff, self.token_file, __event_emitter__)
        except Exception as e:
            print(f"\n\n***ERROR authenticating with Google Calendar API... ***\n{e}\n")
            await event_emitter.error_update(
                f"Error authenticating with Google Calendar API: {e}"
            )
            return {}
        
        await event_emitter.progress_update("Authenticated with Google Calendar API...")
        print(f"\n\n*** Authenticated with Google Calendar API... ***\n{creds}\n")

        try:
            print("\n\n*** Setting up service ....***\n\n")
            await event_emitter.progress_update("Setting up Google Calendar service...")
            try:
                service = build("calendar", "v3", credentials=creds)
                print("\n\n*** Service setup is complete ....***\n\n")
                await event_emitter.progress_update(
                    "Google Calendar service setup complete."
                )
                await event_emitter.progress_update("Getting upcoming events from Google Calendar...")
            except Exception as e:
                print(f"\n\n***ERROR setting up service ... ***\n{e}\n")
                return {}
            # Call the Calendar API
            now = dt.now(tz=datetime.timezone.utc).isoformat()
            print("\n\n*** Getting the upcoming 10 events ***\n\n")
            await event_emitter.progress_update(
                "Getting the upcoming 10 events..."
            )
            events_result = (
                service.events()
                .list(
                    calendarId="primary",
                    timeMin=now,
                    maxResults=10,
                    singleEvents=True,
                    orderBy="startTime",
                )
                .execute()
            )
            events = events_result.get("items", [])
            print(f"\n\n*** EVENTS: ***\n{events}")
            await event_emitter.progress_update(
                f"Retrieved {len(events)} upcoming events from Google Calendar."
            )
            if not events:
                print("No upcoming events found.")
                await event_emitter.error_update("No upcoming events found.")
                return {}

            # Prints the start and name of the next 10 events
            for event in events:
                start = event["start"].get("dateTime", event["start"].get("date"))
                await event_emitter.message_update(
                    f"Upcoming event: {start} - {event['summary']}"
                )
                print(start, event["summary"])

            await event_emitter.success_update("Retrieved upcoming events from Google Calendar.")

        except HttpError as error:
            print(f"An error occurred: {error}")
        return {}


if __name__ == "__main__":
    tools = Tools()
    print(tools.get_current_time())
    print(tools.get_current_weather())
    print(tools.get_geolocation_and_public_ip())
    asyncio.run(tools.get_my_calandar())
    