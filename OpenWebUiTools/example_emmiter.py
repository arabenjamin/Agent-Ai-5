    async def test_function(self, __event_emitter__=None, __event_call__=None) -> str:
        """
        This is a demo

        :param test: this is a test parameter
        """
        try:
            print("\n\n*** Emitting Status message !**\n\n")
            await __event_emitter__(
                {
                    "type": "status",  # We set the type here
                    "data": {
                        "description": "Message that shows up in the chat",
                        "hidden": False,
                        "done": False,
                    },
                    # Note done is False here indicating we are still emitting statuses
                }
            )
            print("\n\n*** Getting User Input !**\n\n")
            result = await __event_call__(
                {
                    "type": "input",  # Or "confirmation", "execute"
                    "data": {
                        "title": "Please enter your password",
                        "message": "Password is required for this action",
                        "placeholder": "Your password here",
                    },
                }
            )

            print("\n\n*** Displaying User input !**\n\n")
            await __event_emitter__(
                {
                    "type": "notification",
                    "data": {"type": "info", "content": f"You entered: {result}"},
                }
            )
            # Do some other logic here
            print("\n\n*** Emitting Task Complete !!**\n\n")
            await __event_emitter__(
                {
                    "type": "status",
                    "data": {
                        "description": "Completed a task message",
                        "done": True,
                        "hidden": False,
                    },
                    # Note done is True here indicating we are done emitting statuses
                    # You can also set "hidden": True if you want to remove the status once the message is returned
                }
            )

        except Exception as e:
            await __event_emitter__(
                {
                    "type": "status",
                    "data": {"description": f"An error occured: {e}", "done": True},
                }
            )

        return f"Tell the user: {e}"