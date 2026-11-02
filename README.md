Matrix Status Bot
===

Simple Bot changing the room topic based on [SpaceAPI](https://spaceapi.io).

Configuration
---

The bot can be configured using environment variables.

| Option                    | Type   | Required? | Description                                                                     | 
|---------------------------|--------|-----------|---------------------------------------------------------------------------------|
| MATRIX_AUTH_TOKEN         | String | Yes       | The Auth Token for the Bot Account                                              |  
| MATRIX_HOMESERVER_ADDRESS | String | Yes       | The address of the homeserver for the Bot Account                               | 
| MATRIX_CHANNEL_ID         | String | Yes       | The ID of the Channel the Bot should manage. It is not resolved from some alias |
| MATRIX_TOPIC_CLOSED       | String | Yes       | Topic when the space is closed                                                  | 
| MATRIX_TOPIC_OPEN         | String | Yes       | Topic when the space is open                                                    | 
| SPACEAPI_ENDPOINT         | String | Yes       | Endpoint for the spaceapi of the ... space                                      |

 