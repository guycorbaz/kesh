# Epics 37, 38 et 39 — numéros réservés

**Décision de Guy (Project Lead), 2026-11-14** : les numéros d'epic **37, 38 et 39** sont réservés à un développeur externe qui
commence à travailler sur Kesh.

## Ce que la réservation implique

- **Aucun autre epic ne prend ces numéros.** Le prochain epic ouvert par l'orchestrateur après l'Epic 36 porte le numéro **40**.
- **Rien n'est créé d'avance** : ni fichier d'epic, ni clé `epic-37` à `epic-39` dans `sprint-status.yaml`, ni story. Chacun
  de ces epics naît à son kickoff, avec son fichier `planning-artifacts/epic-3N-<objet>.md` et ses clés, comme tout epic.
  Les décomptes de clés `epic-<entier>` (`CLAUDE.md`, § *Project Overview*) ne comptent donc que les epics réellement ouverts.
- **Le contenu, l'ordre et le calendrier** de ces trois epics se décident avec le développeur et Guy ; ce fichier ne les
  préjuge pas.
- **Les règles du dépôt s'appliquent** telles qu'elles sont écrites au `CLAUDE.md` (branches, commits en `refs #N`, revues,
  gates, politique des migrations, « on trace toujours »). Kesh tient des livres réels en production : aucune donnée réelle
  dans le dépôt, les issues ou les fiches.

## Coordination

Les epics de l'orchestrateur et ceux du développeur peuvent avancer en parallèle. Deux points se règlent au kickoff de chacun :
les **fichiers communs** (même règle que les stories parallèles : tableau des fichiers communs dans le fichier d'epic) et les
**migrations** (numérotées par horodatage ; l'ordre se contrôle à l'intégration).
