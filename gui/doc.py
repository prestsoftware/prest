from urllib.parse import urljoin
import threading
import webbrowser
import http.server
from typing import Tuple

ADDRESS : Tuple[str, int] = ('127.0.0.1', 8137)

def start_daemon(doc_root : str) -> None:
    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs) -> None:
            kwargs['directory'] = doc_root
            http.server.SimpleHTTPRequestHandler.__init__(self, *args, **kwargs)

    httpd = http.server.HTTPServer(ADDRESS, Handler)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()

    # TODO: try several ports in case the user runs several instances of Prest

def open_in_browser(rel_url : str) -> None:
    host, port = ADDRESS
    url = urljoin(f'http://{host}:{port}/', rel_url)
    webbrowser.open(url, new=2, autoraise=True)  # new=2 -> new tab
