# Proposition : Tableau de Bord VSCode pour la Forge Nebula

## Contexte
L'architecture de distillation de connaissances (Teacher -> Student) s'exécute de manière asynchrone via des FaaS WebAssembly sur le maillage Tachyon. L'utilisateur a besoin d'une interface ergonomique, intégrée à son environnement de développement (VSCode), pour déclencher ces workflows, monitorer la constitution des datasets contrastifs, et visualiser la publication des couches LoRA.

## Objectifs
Développer l'interface utilisateur au sein du package `packages/extension` :

1. **Dashboard Webview** : Une vue riche intégrée à VSCode permettant de visualiser l'état de l'essaim et de la forge.
2. **IPC & Pont WebSocket** : Câbler la communication sécurisée (mTLS) entre l'extension VSCode et le daemon Node.js (CLI) pour faire remonter les événements du bus Tachyon en temps réel.
3. **Contrôle Interactif** : Intégrer les commandes manuelles pour déclencher le `nebula-curriculum-generator` (Apprentissage Zero-Doc) et forcer la fusion d'un modèle (Baking).