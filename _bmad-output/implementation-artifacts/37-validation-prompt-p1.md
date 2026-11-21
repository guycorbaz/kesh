# Prompt de validation P1 — fiches de l'Epic 37 (une fiche par passe)

Tu fais la passe de validation **P1** d'une fiche de story du dépôt `<dépôt>` (Kesh, logiciel
comptable suisse en production : Rust/Axum, SvelteKit, MariaDB). Contexte obligatoire, à lire d'abord :

- le fichier d'epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md` (décisions D1–D7 de l'utilisateur,
  non négociables ; registre C-37-1 à C-37-8, tranché par une table ronde) ;
- la fiche à valider (indiquée dans ta mission) ;
- `CLAUDE.md` à la racine, sections *Migration breaking policy* (P1–P8), *Pattern batch*, *Un appariement automatique
  propose*, *Review Iteration Rule* (pour le format) ;
- le code cité par la fiche : **vérifie chaque prémisse et chaque numéro de ligne au code** (`grep -n`, lecture).

## Ce que tu cherches

1. Prémisse fausse au code (fonction, ligne, comportement, table, colonne qui n'existe pas ou ne fait pas ce que la
   fiche dit).
2. Critère inatteignable, ambigu, contradictoire avec un autre AC, avec une autre fiche de l'epic ou avec le registre.
3. **Sites non résolus** : chemins d'écriture, sélecteurs, exports, sauvegarde, MCP, réinitialisation, i18n, audit,
   manuels que la fiche devrait couvrir et ne nomme pas. Cherche-les au code (énumère, ne suppose pas).
4. Défaut comptable (principes suisses, CO 957–960e) ou de sécurité (réseau sortant, entrée non bornée).
5. Migration : breaking ou non selon P1–P3, inscriptions P5–P8, fenêtre d'import `.keshbackup`.
6. Tests manquants pour un AC.

## Interdits

- **N'écris rien** dans le dépôt, ne crée aucun fichier, ne lance **aucun** script qui écrit
  (`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `cargo`, `npm`, `git commit`…). Lecture seule :
  `cat`, `sed -n`, `grep`, `ls`, `git log/show`.

## Rapport (en français)

- Liste des findings : identifiant `<story>-P1-N`, sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), lieu (AC ou
  section), constat **avec la ligne de code lue citée**, correctif proposé (texte prêt à insérer dans la fiche).
- **Axes exercés / non exercés** (obligatoire : un « 0 finding » sans cette liste ne compte pas).
- Décompte final par sévérité.
