# Prompt de validation P3 — fiches de l'Epic 35 (une fiche par passe)

Tu fais la passe de validation **P3** d'une fiche de story du dépôt `/home/beat/limu/compta/kesh` (Kesh, logiciel
comptable suisse en production : Rust/Axum, SvelteKit, MariaDB). Contexte obligatoire, à lire d'abord :

- le fichier d'epic `_bmad-output/planning-artifacts/epic-35-multi-devises.md` (décisions D1–D7 de l'utilisateur,
  non négociables ; registre C-35-1 à C-35-8, tranché par une table ronde) ;
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

- Liste des findings : identifiant `<story>-P3-N`, sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), lieu (AC ou
  section), constat **avec la ligne de code lue citée**, correctif proposé (texte prêt à insérer dans la fiche).
- **Axes exercés / non exercés** (obligatoire : un « 0 finding » sans cette liste ne compte pas).
- Décompte final par sévérité.

## Spécifique à la P3

La fiche a été remédiée après une passe P1 (voir son Change Log) et les six fiches de l'epic ont été alignées entre
elles (registre C-35-9 du fichier d'epic). Cherche en priorité : (a) les défauts **introduits par la remédiation**
(contradictions internes, numéros de ligne faux, exigences irréalisables, renvois à des AC renumérotés) ; (b) les
**incohérences entre fiches** (codes d'erreur, statuts HTTP, noms de fonctions, ordre de dépendance, qui fait quoi) —
lis les cinq autres fiches `35-*.md` du même dossier pour les points de contact ; (c) ce que la P1 n'a pas vu.
