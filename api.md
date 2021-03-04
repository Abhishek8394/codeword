# API
| Method | Route | Desc | Req. Data |
| --- | --- | --- | --- |
| POST | `/lobby` | Create a lobby. | |
| POST | `/lobby/<id>/players` | Join lobby as player. | player id |
| GET | `/lobby/<id>/players` | Get lobby players info. | |
| POST | `/lobby/<id>/joinTeam` | Join a team. | player id, team id |
| POST | `/lobby/<id>/becomeSpymaster` | Become spymaster. | player id |
| GET | `/lobby/<id>/gameInfo` | Get full game state. Depends on if caller a player or spymaster. | player id |
| GET | `/lobby/<id>/gameUpdate` | Get minimal game state update. | player id |
| POST | `/lobby/<id>/unravel/` | Send unravel request. Can only do when your team's turn and you not a spymaster. | player id |
| GET | `/lobby/<id>/actionLogs/` | Get action logs for game. | player id |
| POST | `/lobby/<id>/hints` | Create a hint. | hint |
| GET | `/lobby/<id>/hints/last` | Get last hint. | hint |

## Optional
| PUT | `/lobby/<id>/ready` | Signal player ready. | player id |

## Alt Arch
| GET | `/lobby/<id>/gameviews/player` | Get game state from player perspective. PlainBoard, score, team catalogs, score, turn, state. | player id |
| GET | `/lobby/<id>/gameviews/spymaster` | Get game state from spymaster perspective. FullBoard, score, team catalogs, score, turn, state. | player id |
