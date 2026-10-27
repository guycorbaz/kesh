# Revue de code P1 — Story 32-7 (l'écran « Serveur MCP » et la section du manuel)

Prompt versionné, commun aux trois lentilles de la passe 1. **Passe complète.** Modèle : **Opus**, contexte frais,
lecture seule. Une lentille par agent : **B**, **E** ou **A**.

## Objet

Worktree : `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-7`, `main` fusionnée au commit `e901a992` (les 32-6a à
32-10b y sont). Diff à relire, **aplati** : `git -C /home/gcorbaz/devel/kesh-32-10a diff e901a992...HEAD` (pas la suite
des commits). Fiche : `_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` (25 AC, § *Dev Agent Record*
et *Change Log* pour ce qui a tourné) ; registre `epic-32-choix-autonomes-327.md` (C-32-7-1 à C-32-7-33 ; les choix du
développement sont C-32-7-28 à 33 — à contester s'ils sont faux).

Ce que la story livre :

- l'écran `/admin/mcp` (`frontend/src/lib/features/mcp-server/`, `frontend/src/routes/(app)/admin/mcp/{+page.ts,
  +page.svelte}`) : réglages du serveur (activation, filtre d'adresse, plages, mandataires, origines), avertissements
  informatifs, encadré « Ce que le filtre ne garantit pas » ; clés MCP (liste, création, secret affiché une fois par
  une instance de `SecretUnique`, révocation, politique de validation) ; entrée de menu `nav-admin-mcp` ;
- 96 clés i18n dans les quatre catalogues (`crates/kesh-i18n/locales/*/messages.ftl`, fin de fichier) ;
- gardes : `mcp-server-gardes.test.ts` (G-a, G-b), extension de G13 par préfixe (`i18n-repli-divergent-actif.test.ts`,
  C-32-7-28), `i18n-keys.test.ts` (`'admin-mcp'`, `sitesTotal` 2513 → 2617), `i18n-libelle-en-dur.test.ts`
  (`CLASSES_ADMISES`, C-32-7-29), G23 et G18 amendée dans `crates/kesh-api/tests/textes_coherents.rs` ;
- `\section{Le serveur MCP}\label{sec:mcp}` du manuel d'administration (après *Sécurité*, avant *Monitoring*), ses
  sous-sections neuves (`sec:mcp-activer`, `-cles`, `-validation`, `-filtre`, `-origin`, `-journal`, `-erreurs`,
  `-client`), les trois sous-sections déplacées sans changer d'étiquette (`sec:mcp-outils`, `-envois`, `-prompts`),
  les renvois (AC-19) au manuel d'administration et au manuel utilisateur, les deux PDF régénérés, le CHANGELOG ;
- l'E2E `frontend/tests/e2e/mcp-server.spec.ts`.

**Captures** (thème clair), à regarder : `/home/gcorbaz/devel/kesh-gate-logs/327-captures/` (30 fichiers, nommés par
numéro de l'AC-17). **Journaux** : `/home/gcorbaz/devel/kesh-gate-logs/327/` (rouges constatés, mutations
`327-mut-M<n>.log`, gates, motifs PDF).

**Manuel** — à contrôler contre le CODE, et sur le **PDF**, pas seulement le `.tex` :
`pdftotext /home/gcorbaz/devel/kesh-32-10a/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et
`user-manual.pdf`). Chaque affirmation de la section *Le serveur MCP* doit être vraie au code :
`crates/kesh-api/src/mcp/` (`transport.rs`, `reseau.rs`, `reponse.rs`, `textes.rs`, `repartiteur.rs`, `mod.rs`),
`crates/kesh-api/src/services/{cles_mcp,serveur_mcp,envois_mcp,prompts_mcp}.rs`, `crates/kesh-api/src/routes/
{mcp_keys,mcp_server_settings,users,onboarding}.rs`, `crates/kesh-db/src/{backup.rs,entities/mcp_key.rs,
entities/mcp_server_settings.rs,repositories/mcp_server_settings.rs,repositories/mcp_keys.rs}`, `crates/kesh-api/src/
mcp/outils/` (portées et masquage), et `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 2.1, 2.4, 6) pour
la configuration de Claude Code. Le précédent : sur l'Epic 24, c'est en vérifiant si le manuel disait vrai qu'une
lentille a trouvé les défauts les plus chers.

## ⛔ Interdits

- **N'exécute JAMAIS `scripts/prepare-release.sh`**, ni directement ni par aucun détour. N'exécute aucun script du
  dépôt (`scripts/test-fast.sh`, `scripts/mem-guard.sh`…), ni `cargo`, `npm`, `npx`, `node`, `python3`, `sqlx`,
  `docker`, `mariadb`/`mysql`, `make`/`latexmk`, `curl`.
- **Aucune redirection** (`>`, `>>`, `<`, `tee`, here-doc) : compare par des commandes enchaînées (pipe) et lis vers la
  sortie standard seulement. **Ni `sed -i`, ni `perl -pi`, ni `rm` sous aucune forme.**
- N'écris, ne modifie, ne crée aucun fichier — **sauf ton rapport**, écrit avec l'outil d'écriture de fichiers. Aucune
  commande git qui écrit, **aucun `git checkout`/`git switch` sous aucune forme**, ni `stash`, `reset`, `rebase`, `tag`,
  `push` ; aucune commande `gh` qui écrit. Aucun `pkill`/`kill`.
- N'ouvre jamais `~/kesh-test/`.
- Commandes autorisées : `git diff`, `git show`, `git log`, `git ls-tree`, `grep` (sur ce poste `grep` est une fonction
  `ugrep` : employer `command grep`), `sed -n`, `cat`, `pdftotext` (vers `-`), `tr`, lecture de fichiers et d'images.

## Lentille (une seule par agent)

- **B — Blind Hunter** : lis le diff comme un relecteur qui ne connaît pas le dépôt. Le secret : peut-il survivre à
  *Fermer*, à un 400, à un rechargement, dans un autre `$state`, un attribut, un journal, un toast, le stockage,
  l'adresse ? `createMcpKey` construit-il la clé champ par champ ? Les dates (`depuisUtcNaif`, fin de journée locale)
  sont-elles justes, et réaffichées identiques ? Les avertissements (`aPrefixeNul`, `familleDe`) disent-ils vrai ? Les
  tests prouvent-ils ce qu'ils annoncent — peuvent-ils passer à vide (espion sans témoin, `waitFor` qui passe avant le
  chargement, mock qui masque la propriété) ? Traductions de/it/en : contresens ? DRY (le module partagé
  `i18n-repli-catalogue.ts`, `errorMessageOf`, `copyToClipboard`).
- **E — Edge Case Hunter** : chaque branche et chaque bord. Au minimum : réglages à `version: 0`, ligne corrompue
  (500 au `GET`), 409, 400, panne réseau, double clic sur *Enregistrer* ; plages vides, lignes blanches, `/0`, `/00`,
  `::/0`, adresse mappée, entrée sans préfixe ; filtre désactivé avec mandataire `/0` ; clé sans portée connue ; clé
  expirée **et** révoquée ; `expiresAt` à la frontière de minuit, changement d'heure, fuseau autre que Zurich ; nom de
  255 / 256 points de code, nom fait de blancs invisibles ; deux révocations concurrentes ; politique changée pendant
  que la confirmation est ouverte ; création pendant qu'un secret est affiché ; un 400 après un 201 ; navigation hors
  de la page pendant un envoi. Le menu : ouverture automatique du groupe sur `/admin/mcp` ; le Comptable.
- **A — Acceptance Auditor** : confronte le livrable aux 25 AC de la fiche, un par un, avec preuve (ligne citée) ; puis :
  1. le **§ *À recontrôler à l'étape 0*** de la fiche et sa note des **réponses de Guy** (Hermes retiré, un seul
     client nommé ; créateur non affiché ; renvoi à `sec:mcp-journal-filtre`) : appliqués ?
  2. les **écarts déclarés** au Dev Agent Record (phrases de portée réécrites selon le code, C-32-7-31 ; « un panic
     n'est pas journalisé » contredit par `repartiteur.rs:564-575` ; motif PDF `Le serveur MCP` remplacé ; capture 24) :
     justes ?
  3. les **mutations** M1 à M26 : chacune tuée par le test que la fiche vise (journaux `327-mut-M<n>.log`) ?
  4. les gardes : G23 (blocs extraits du source brut, témoins), G-a/G-b (périmètre exact, 11 fichiers), G13 par
     préfixe (comptes 95/48/94), `sitesTotal` 2617 recompté, G18 (21 occurrences, trois fragments neufs assumés) ;
  5. commits `refs #723`, **aucun mot-clé de fermeture** :
     `git -C /home/gcorbaz/devel/kesh-32-10a log e901a992..HEAD --format=%B | command grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
     doit ne rien rendre ;
  6. aucune donnée réelle, aucun secret réel dans le dépôt (`kesh_mcp_` suivi de 27 caractères base62 : `command grep
     -rnE 'kesh_mcp_[0-9A-Za-z]{27}'` sur le diff).

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/327-review-p1-<B|E|A>.md` (selon ta lentille) :

1. **Axes exercés** et **axes NON exercés** — liste explicite. Un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `P1-<B|E|A>-n`, chacun avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne,
   description, preuve, correctif proposé. **Toute affirmation de présence ou d'absence d'un code s'appuie sur une
   sortie `command grep -nF` copiée dans le rapport** ; toute affirmation sur le manuel, sur le PDF aplati. Un défaut
   préexistant, hors du diff, se signale comme tel (« hors périmètre, à tracer »).
3. Verdict : nombre de findings par sévérité.
