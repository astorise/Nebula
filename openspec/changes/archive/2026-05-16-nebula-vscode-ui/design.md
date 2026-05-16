# Design : Tableau de bord VS Code

## Architecture

Le dashboard est separe du host VS Code :

- `NebulaDashboardProvider` gere le panneau webview, l'etat applicatif, le message passing VS Code et le relais WebSocket mTLS.
- `src/webview/main.ts` contient l'interface compilee par esbuild vers `media/webview.js`.
- `messages.ts` formalise les enveloppes `EVENT|COMMAND` et l'etat partage.

## Flux

Le webview envoie les commandes utilisateur via `acquireVsCodeApi().postMessage()`. Le provider les normalise en enveloppes `{ type: "COMMAND", action, payload }` et les transmet au CLI par WSS.

Les evenements WSS entrants mettent a jour l'etat local :

- `nebula.dataset.append` incremente la jauge dataset et le ratio 60/40.
- `nebula.eval.results` alimente le journal de divergences.
- `nebula.training.ready` et `nebula.training.complete` pilotent l'etat LoRA.

## UI

L'interface reste native VS Code en utilisant les variables de theme. Elle expose une jauge dataset, le formulaire curriculum, les etapes LoRA et un flux de logs Tier 3.
