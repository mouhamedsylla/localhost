#!/usr/bin/env python3
import os
import sys
import json
from datetime import datetime

def main():
    # Vérifier que c'est une requête GET
    request_method = os.environ.get('REQUEST_METHOD', '')
    if request_method != 'GET':
        print_response(405, "Method Not Allowed", "text/plain", "Only GET method is supported")
        return

    # Récupérer le chemin de la requête
    script_name = os.environ.get('SCRIPT_NAME', '')
    
    # Préparer les données de réponse
    response_data = {
        "timestamp": datetime.now().isoformat(),
        "method": request_method,
        "path": script_name,
        "server_software": os.environ.get('SERVER_SOFTWARE', ''),
        "server_protocol": os.environ.get('SERVER_PROTOCOL', ''),
        "query_string": os.environ.get('QUERY_STRING', ''),
        "remote_addr": os.environ.get('REMOTE_ADDR', ''),
        "http_headers": {
            k[5:].lower().replace('_', '-'): v 
            for k, v in os.environ.items() 
            if k.startswith('HTTP_')
        }
    }

    # Envoyer la réponse au format JSON
    print_response(
        200,
        "OK",
        "application/json",
        json.dumps(response_data, indent=2)
    )

def print_response(status, status_text, content_type, body):
    """Envoie une réponse HTTP formatée"""
    print(f"Status: {status} {status_text}")
    print(f"Content-Type: {content_type}")
    print(f"Content-Length: {len(body)}")
    print("X-Powered-By: Python CGI")
    print("\r\n\r\n")  # Ligne vide requise entre les headers et le body
    print(body)

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print_response(
            500,
            "Internal Server Error",
            "text/plain",
            f"CGI Error: {str(e)}"
        )