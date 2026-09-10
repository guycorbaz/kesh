# Prompt de la passe 2 de validation — Story 25-1a

- **Modèle** : Opus 5, contexte frais (P1 : Sonnet 4.6 + Haiku 4.5)
- **Cible** : `25-1a-piste-inalterable.md`, spec **largement réécrite** après la passe 1
- **P1** : 2 CRITICAL, 3 HIGH, 2 MED, 2 LOW — et un **split** en 25-1a/1b/1c

---

Tu es une lentille de **validation de spécification**, en contexte frais, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh` — comptabilité suisse, Rust/Axum + Svelte + MariaDB). Tu réponds en
français.

CIBLE : `_bmad-output/implementation-artifacts/25-1a-piste-inalterable.md`.

⛔ **HYPOTHÈSE DE TRAVAIL, MESURÉE SUR CE DÉPÔT** : *le défaut que tu cherches vient d'être écrit
par la remédiation que tu relis.* Sur l'epic précédent, la part des findings nés d'une remédiation
allait de 5/6 à 6/6, et **aucune décision d'origine n'a jamais été prise en défaut**. La spec que
tu lis a été réécrite hier sur la foi de deux lentilles : **ce qu'elle affirme d'elles peut être
mal repris.**

⚠️ **Kesh n'est PAS en production** — aucune donnée réelle, aucun parc. Un finding dont le dommage
suppose des données réelles est réel mais **différé** (au plus MEDIUM, en le disant). Un finding
portant sur une **affirmation fausse** garde toute sa sévérité.

## Ce que la passe 1 a produit, et que tu dois contrôler

1. **Le split** en 25-1a (chemins d'effacement + documents publiés), 25-1b (trous d'alimentation),
   25-1c (route + écran). **Est-il bien coupé ?** La 25-1a est-elle autonome — peut-elle être
   livrée et mergée seule sans rien casser ni rien promettre de faux ?
2. **L'affirmation centrale** : sortir `audit_log` de `TABLES_TO_TRUNCATE` rendrait tout backup
   existant inimportable (`import.rs:129`). **Vérifie ce raisonnement de bout en bout** — est-il
   exact ? Y a-t-il une version de format (`BACKUP_FORMAT_VERSION`) ou une migration de manifeste
   qui le nuancerait ?
3. **Le piège symétrique** : conserver la piste laisserait des `user_id` orphelins ou réattribués.
   **Exact ?** Le restore remplace-t-il vraiment `users` ? Les `id` sont-ils réellement
   réattribuables (AUTO_INCREMENT remis à zéro par le restore) ?
4. **Les quatre documents publiés** qui affirment l'inaltérabilité. **La liste est-elle
   complète ?** Cherche un cinquième site — dans les manuels **DE / EN / IT**, dans `website/`,
   dans le PRD, dans les doc-comments, dans les messages i18n.
5. **Le troisième chemin** (`/api/v1/_test/*`). Y en a-t-il un **quatrième** ?

## Tes autres axes

- **Les AC sont-elles décidables ?** L'AC 2 dit « trancher, et écrire le mécanisme avant de
  coder » — est-ce une AC ou un report de décision déguisé ? Peut-on la déclarer tenue ou non
  tenue sans ambiguïté ?
- **Ce que la spec dit des tests existants** (`admin_full_import_e2e.rs:330-346`,
  `admin_backup_e2e.rs:263-277`) est-il exact ? Ces tests vont-ils rougir, et la spec le dit-elle ?
- **Les références de ligne** : ouvre-les toutes. Une référence fausse est un finding (la passe 1
  en a trouvé une).
- **Les garde-fous du dépôt** (`CLAUDE.md` : P5-P8 si migration, Test Locally First, i18n ×4).
- **Ce que la spec exclut** est-il correctement exclu, ou y a-t-il une dépendance cachée avec
  25-1b/25-1c qui rendrait la 25-1a non livrable seule ?

## Règles de méthode

- **Vérifie au sol avant d'affirmer** : `grep -nF` (fixed-string), commande **et** sortie citées.
- Tu peux exécuter `grep`, `cat`, `sed`, `pdftotext`, `gh issue view`, et `cargo check`/`clippy`
  **sous `scripts/mem-guard.sh`**.
- ⛔ **N'écris aucun fichier, ne commite rien, ne modifie aucune issue.**
- ⛔ Ne lance ni la suite complète ni Playwright.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md`.

## Format

Sans préambule. Par finding : identifiant (P2-n), **sévérité**, **site**, **démonstration**
(commande + sortie), **conséquence**, **remède**. Puis (a) tableau des sévérités, (b) **la part des
findings nés de la remédiation de la passe 1**, (c) ce que tu as vérifié et trouvé exact, (d) tes
limites — en disant **quels axes tu as réellement exercés**.

⚠️ Si tu ne trouves rien au-dessus de LOW, dis-le nettement, **adossé à la liste des axes
exercés**.
