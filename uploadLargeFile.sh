#!/bin/bash

# Définir l'URL du serveur
SERVER_URL="http://uploader.home:8080/api/files/upload"

# Fichier à uploader
FILE_PATH="./largeFile.txt"

# Vérifier si le fichier existe
if [[ ! -f "$FILE_PATH" ]]; then
  echo "Fichier $FILE_PATH introuvable."
  exit 1
fi

# Effectuer la requête curl avec Transfer-Encoding: chunked
curl -X POST "$SERVER_URL" \
     -H "Transfer-Encoding: chunked" \
     -F "file=@$FILE_PATH"

# Vérifier le code de sortie
if [[ $? -eq 0 ]]; then
  echo "Upload réussi !"
else
  echo "Échec de l'upload."
fi


