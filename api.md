# API
| Method | Route | Desc | Req. Data |
| --- | --- | --- | --- |
| POST | `/lobby` | Create a lobby. | |
| POST | `/lobby/<id>/players` | Join lobby as player. | player id |
| POST | `/lobby/<id>/joinTeam` | Join a team. | player id, team id |
| POST | `/lobby/<id>/becomeSpymaster` | Become spymaster. | player id |
| PUT | `/lobby/<id>/ready` | Signal player ready. | player id |
| GET | `/lobby/<id>/gameviews/player` | Get game state from player perspective. PlainBoard, score, team catalogs, score, turn, state. | player id |
| GET | `/lobby/<id>/gameviews/spymaster` | Get game state from spymaster perspective. FullBoard, score, team catalogs, score, turn, state. | player id |
| POST | `/lobby/<id>/unravel/` | Send unravel request. Can only do when your team's turn and you not a spymaster. | player id |
| GET | `/lobby/<id>/actionLogs/` | Get action logs for game. | player id |
