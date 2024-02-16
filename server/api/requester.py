import requests

from pyld import jsonld


def fetch_remote_id(id: str):
    res = requests.get(id)
    j = res.json()
    return jsonld.expand(j)
