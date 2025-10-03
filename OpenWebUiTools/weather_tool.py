import os
import requests
import logging

from typing import Any, Callable, List

from pydantic import BaseModel, Field

numeric_log_level = getattr(logging, "DEBUG", None)
if not isinstance(numeric_log_level, int):
    raise ValueError('Invalid log level: %s' % "DEBUG")

# Set up logging
logger = logging.getLogger(__name__)
logging.basicConfig(

    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    filename='WeatherToolLog.log', 
    encoding='utf-8', 
    level=numeric_log_level
)


class EventEmitter:
    def __init__(self,  event_emitter: Callable[[dict], Any] = None, call_emitter: Callable[[dict], Any] = None):
        
        logger.info("Initializing EventEmitter")
        if not callable(event_emitter):
            raise ValueError("event_emitter must be a callable function.")
        self.event_emitter = event_emitter
        logger.info("\n\n*** Emitter initiated  ***\n\n")

    async def progress_update(self, description: str):
        """Emit a progress update."""
        logger.info(f"\n\n*** Emitting progress update: {description}  ***\n\n")
        await self.emit(description)

    async def error_update(self, description: str):
        """Emit an error update and mark as done."""
        logger.error(f"\n\n*** Emitting error!!: {description}  ***\n\n")
        await self.emit(description, "error", True)

    async def message_update(self, content: str):
        """Emit a message update."""
        logger.info(f"\n\n*** Sending message To frontend: {content}  ***\n\n")
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
        logger.info(f"\n\n***Success!: {description}  ***\n\n")
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
        logger.info(f"***Emitting message: {description}***")
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

    def __init__(self, __event_emitter__: Callable[[dict], Any], __event_call__: Callable[[dict], Any] = None):
        self.valves = self.Valves()
        self.event_emitter = __event_emitter__
        self.emitter = EventEmitter(__event_emitter__, __event_call__)

    async def _get_geocoords(self, api_key: str, country: str, zipcode: str) -> dict:
        
        logger.info("Getting Geo Coordintes for weather look up")
        await self.emitter.progress_update(
            f"Fetching geocoordinates for {zipcode}, {country}"
        )
        url = f"http://api.openweathermap.org/geo/1.0/zip?zip={zipcode},{country}&appid={api_key}"
        resp = requests.get(url)
        logger.info(f"Coords Response: {resp.status_code}")

        if resp.status_code != 200:
            await self.emitter.error_update(
                f"Error fetching geocoordinates: {resp.status_code} - {resp.text}"
            )
            logger.error(f"Error fetching geocoordinates: {resp.status_code} - {resp.text}")
            raise Exception(f"Error fetching geocoordinates: {resp.status_code} - {resp.text}")
        if resp.status_code == 200:
            await self.emitter.success_update(
                f"Successfully fetched geocoordinates for {zipcode}"
            )
            logger.info(f"Successfully fetched geocoordinates for {zipcode}")
            return resp.json()
        return {}

    async def get_weather_forcast(self, ) -> str:
        """Get weather information for a given location."""
        try:
            await self.emitter.progress_update(f"Fetching weather for Bothell, WA")
            api_key = os.getenv("WEATHER_API_KEY")
            if not api_key:
                raise ValueError("WEATHER_API_KEY environment variable is not set.")

            try:
                await self.emitter.progress_update(f"Fetching geocoordinates for Bothell, WA")
                coords = await self._get_geocoords(api_key, "US", "98012")
            except Exception as e: 
                await self.emitter.error_update(f"Error fetching geocoordinates: {str(e)}")
                return "Error fetching geocoordinates: " + str(e)

            await self.emitter.progress_update(f"Coordinates for Bothell, WA: {coords}")
            logger.info(f"Coordinates for Bothell, WA: {coords}")
            await self.emitter.progress_update(f"Fetching weather data for Bothell, WA")
            url = f"https://api.openweathermap.org/data/3.0/onecall?lat={coords['lat']}&lon={coords['lon']}&appid={api_key}"
            response = requests.get(url)
            response.raise_for_status()
            data = response.json()

            await self.emitter.success_update(f"Weather fetched successfully for Bothell, WA", data)
            return f"Weather data for {coords['name']}: {data}"

        except Exception as e:
            await self.emitter.error_update(f"Error fetching weather for Bothell, WA: {str(e)}")
            return "Error fetching weather: " + str(e)
        
    async def get_current_weather(self, __event_emitter__: Callable[[dict], Any] = None, __event_call__: Callable[[dict], Any] = None) -> str:
        """Get the current weather for a given zipcode."""
        logger.info("\n\n***GETTING CURRENT WEATHER***\n\n")
        
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
            coords = await self._get_geocoords(self.valves.api_key, "US", zipcode)
            await event_emitter.progress_update(
                f"Found Geolocation coordinates LAT: {coords['lat']}, LONG: {coords['lon']} "
            )
            logger.info(
                f"\n\n*** COORDS: LAT: {coords['lat']}, LONG: {coords['lon']} ***\n\n"
            )
        except Exception as e:
            logger.info(
                f"***ERROR: There was an error retieving Coordinates for the location *** \n{e}\n"
            )
            return f"Error retrieving coordinates: {str(e)}"

        await event_emitter.progress_update(
            f"Fetching weather data for coordinates: LAT: {coords['lat']}, LONG: {coords['lon']}"
        )

        try:
            weather_url = f"https://api.openweathermap.org/data/2.5/weather?lat={coords['lat']}&lon={coords['lon']}&appid={self.valves.api_key}&units=imperial"
            response = requests.get(weather_url)
            response.raise_for_status()  # Raise HTTPError for bad responses (4xx and 5xx)
            data = response.json()
            logger.info(f"\n\n***CURRENT WEATHER***\n{data}\n")
            await event_emitter.progress_update(
                f"Received weather data for {zipcode}."
            )
            if data.get("cod") != 200:
                await event_emitter.error_update(
                    f"Error fetching weather data: {data.get('message')}"
                )
                return f"Error fetching weather data: {data.get('message')}"

            weather_description = data["weather"][0]["description"]
            temperature = float(data["main"]["temp"])
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
