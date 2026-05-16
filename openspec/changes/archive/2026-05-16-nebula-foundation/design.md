# Design : Fondation Nebula

## Architecture

Le depot est initialise comme monorepo npm avec deux workspaces :

- `packages/cli` expose le moteur local Nebula.
- `packages/extension` expose l'interface VS Code.

Le CLI demarre un serveur HTTPS unique configure en mTLS. Le serveur traite les requetes WebDAV de lecture sur le chemin racine et attache un pont WebSocket sur `/ws`.

## CLI

Le serveur WebDAV resout toutes les requetes dans `NEBULA_DOCS_ROOT` et refuse toute sortie de ce repertoire. Les methodes autorisees sont `GET`, `HEAD`, `OPTIONS` et `PROPFIND`; les methodes modificatrices retournent `405 Method Not Allowed`.

Le pont WebSocket utilise `ws` et verifie que le socket TLS client est autorise. Les messages recus sont transmis a un routeur Tachyon stub, qui renvoie un evenement de routage stable en attendant l'IPC Tachyon reel.

## Extension VS Code

L'extension declare la commande `Nebula: Open Dashboard`. Elle ouvre un webview de pilotage et cree un client WebSocket `wss` avec les chemins de certificats declares dans la configuration `nebula.tls.*`.

## Securite

Les certificats serveur, client et CA sont fournis par configuration locale. Le CLI impose `requestCert: true` et `rejectUnauthorized: true`; l'extension impose aussi la validation du serveur via la CA configuree.
