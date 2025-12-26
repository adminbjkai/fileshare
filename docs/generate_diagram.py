#!/usr/bin/env python3
"""Generate architecture diagram for FileShare application."""

from diagrams import Diagram, Cluster, Edge
from diagrams.onprem.client import User, Client
from diagrams.onprem.network import Nginx
from diagrams.programming.language import Rust
from diagrams.onprem.container import Docker
from diagrams.generic.storage import Storage
from diagrams.generic.compute import Rack
from diagrams.custom import Custom
import os

# Set output directory
os.chdir(os.path.dirname(os.path.abspath(__file__)))

graph_attr = {
    "bgcolor": "#1a1a2e",
    "fontcolor": "#ffffff",
    "fontsize": "14",
    "pad": "0.5",
    "splines": "ortho",
}

node_attr = {
    "fontcolor": "#ffffff",
    "fontsize": "11",
}

edge_attr = {
    "color": "#6c7086",
    "fontcolor": "#a6adc8",
    "fontsize": "10",
}

with Diagram(
    "FileShare Architecture",
    filename="architecture_diagram",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
    outformat="png",
):
    user = User("Browser\nClient")

    with Cluster("Internet", graph_attr={"bgcolor": "#16213e", "fontcolor": "#cdd6f4"}):
        nginx = Nginx("Nginx\nReverse Proxy\nfileshare.bjk.ai:443")

    with Cluster("Server (Port 7012)", graph_attr={"bgcolor": "#0f3460", "fontcolor": "#cdd6f4"}):
        with Cluster("Actix-web Application", graph_attr={"bgcolor": "#1a1a2e", "fontcolor": "#89b4fa"}):
            rust_app = Rust("Actix-web\nServer")

        with Cluster("Handlers", graph_attr={"bgcolor": "#1a1a2e", "fontcolor": "#a6e3a1"}):
            upload = Rack("POST /upload\nFile Handler")
            download = Rack("GET /download\nFile Streamer")
            qr = Rack("GET /qr\nQR Generator")

        with Cluster("Modules", graph_attr={"bgcolor": "#1a1a2e", "fontcolor": "#f9e2af"}):
            zip_mod = Rack("ZIP\nCompressor")
            qr_mod = Rack("QR Code\nEncoder")

    with Cluster("Storage", graph_attr={"bgcolor": "#16213e", "fontcolor": "#f38ba8"}):
        storage = Storage("./uploads/\nFile Storage")
        counter = Storage(".zipcount\nID Counter")

    # Connections
    user >> Edge(label="HTTPS", color="#89dceb") >> nginx
    nginx >> Edge(label="HTTP:7012", color="#89dceb") >> rust_app

    rust_app >> Edge(color="#a6e3a1") >> upload
    rust_app >> Edge(color="#a6e3a1") >> download
    rust_app >> Edge(color="#a6e3a1") >> qr

    upload >> Edge(color="#f9e2af") >> zip_mod
    qr >> Edge(color="#f9e2af") >> qr_mod

    upload >> Edge(label="write", color="#f38ba8") >> storage
    download >> Edge(label="read", color="#f38ba8") >> storage
    zip_mod >> Edge(label="read/write", color="#f38ba8") >> counter

print("Diagram generated: architecture_diagram.png")
