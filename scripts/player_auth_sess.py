import argparse
import json
import requests

def send_create_game_request(addr):
    url = f"http://{addr}/lobby"
    resp = requests.post(url)
    return resp

def send_connect_player_request(sess, addr, lobby_id, name):
    url = f"http://{addr}/lobby/{lobby_id}/players"
    payload = {'name': name}
    resp = sess.post(url, data=json.dumps(payload))
    return resp

def get_game_view(sess, addr, lobby_id):
    url = f"http://{addr}/lobby/{lobby_id}/game_info"
    print(url)
    print(sess.cookies)
    resp = sess.get(url)
    return resp

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--addr', help='web app addr', default='127.0.0.1:8080')
    args = parser.parse_args()

    addr = args.addr
    resp = send_create_game_request(addr)
    lobby_id = resp.text
    print("lobby:", resp.status_code, lobby_id)
    sess_1 = requests.session()
    sess_2 = requests.session()
    player_1 = send_connect_player_request(sess_1, addr, lobby_id, "p1")
    print("p1:", player_1.json(), player_1.headers)
    player_2 = send_connect_player_request(sess_2, addr, lobby_id, "p2")
    print("p2:", player_2.json(), player_2.headers)

    # this should get resp cuz it has cookie
    p1_game_view = get_game_view(sess_1, addr, lobby_id)
    print("p1:", p1_game_view.status_code, p1_game_view.text)
    sess_3 = requests.session()
    # this should get rekt
    p2_game_view = get_game_view(sess_3, addr, lobby_id)
    print("p2:", p2_game_view.status_code, p2_game_view.text)


if __name__ == '__main__':
    main()
