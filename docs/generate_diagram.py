#!/usr/bin/env python3
"""Generate architecture diagram for FileShare application."""

from diagrams import Diagram, Cluster, Edge
from diagrams.onprem.client import User
from diagrams.onprem.network import Nginx
from diagrams.programming.language import Rust
from diagrams.onprem.database import PostgreSQL
from diagrams.programming.framework import FastAPI
from diagrams.onprem.inmemory import Redis
from diagrams.aws.storage import S3
from diagrams.generic.blank import Blank
import os

# Set output directory
os.chdir(os.path.dirname(os.path.abspath(__file__)))

graph_attr = {
    "bgcolor": "#0d1117",
    "fontcolor": "#c9d1d9",
    "fontsize": "12",
    "pad": "0.3",
    "ranksep": "0.5",
    "nodesep": "0.4",
}

node_attr = {
    "fontcolor": "#c9d1d9",
    "fontsize": "10",
}

edge_attr = {
    "color": "#8b949e",
    "fontcolor": "#8b949e",
    "fontsize": "9",
}

with Diagram(
    "",
    filename="architecture_diagram",
    show=False,
    direction="TB",
    graph_attr=graph_attr,
    node_attr=node_attr,
    edge_attr=edge_attr,
    outformat="png",
):
    user = User("Client")

    with Cluster(""):
        nginx = Nginx("Nginx :443")

    with Cluster("Actix-web :7012"):
        rust_app = Rust("Server")

    with Cluster("Endpoints"):
        upload = FastAPI("POST /upload")
        download = FastAPI("GET /{file}")
        qr = FastAPI("GET /qr/{file}")

    with Cluster("Storage"):
        storage = S3("./uploads/")

    user >> Edge(label="HTTPS") >> nginx
    nginx >> Edge(label="proxy") >> rust_app

    rust_app >> upload
    rust_app >> download
    rust_app >> qr

    upload >> Edge(label="write") >> storage
    download >> Edge(label="read") >> storage

print("Diagram generated: architecture_diagram.png")
