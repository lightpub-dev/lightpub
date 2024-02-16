import requests

from pyld import jsonld


class Requester:
    def fetch_remote_id(self, id: str):
        res = requests.get(id)
        j = res.json()
        return jsonld.expand(j)

    def fetch_remote_user(self, id: str):
        pass

    def send_follow_accept(self) -> None:
        pass


_req = Requester()


def get_requester() -> Requester:
    return _req
