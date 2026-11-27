# Prompt — revue de code P2 de la story `64-1-corrections-0201` (#1178, #1758, #1712), passe complète

Tu conduis, **en lecture seule**, la revue de code P2 de la story `64-1-corrections-0201` de Kesh (comptabilité suisse ; Rust/Axum,
MariaDB, SvelteKit 5 runes), seule story de l'Epic 64, **sans migration**. Kesh est en production : le Project Lead y tient ses
livres et y émet de vraies factures. La story porte trois corrections :

- **#1178** : le taux de TVA part en chaîne décimale (`decimalOf`), et un validateur (`validateRateInput`, `validateDateInput`)
  arrête les cas évidents **avant toute écriture** ; au changement de taux, un drapeau distingue l'échec du `PUT` (rien n'est
  clos) de celui du `POST` qui suit la clôture ;
- **#1758** : la page `/invoices/{id}/edit` affiche un avis pour une facture validée ou annulée, et un avis de rôle pour
  Consultation ; elle ne rend jamais le formulaire dans ces deux cas ;
- **#1712** : `PUT /api/v1/profile/mode` passe dans `admin_routes`, et l'entrée « Mode » du menu utilisateur n'apparaît qu'à
  l'Administrateur.

Cadre de la passe :

- **Modèle : Sonnet** (la P1 était Opus). **Contexte frais. Passe complète**, et non ciblée : la P1 a rendu 2 MEDIUM, et leur
  remédiation peut toucher un module de plus (formulaire de facture, catalogue). Langue : français.
- Worktree `kesh-64-1`, branche `story/64-1-corrections-0201`. Base `0e99cd2db` ; reprise de `main` : `e1ca14e76` (diff combiné à
  relire si la remédiation reprend `main` une nouvelle fois). Le diff propre à la story se lit par `git diff e1ca14e76 HEAD` ; la
  remédiation de la P1 par `git log --oneline cf78911c7..HEAD`.
- **Commits de la remédiation de la P1** : `c1d280981` (R1-1 : valeurs de taux dédoublonnées dans `InvoiceForm.svelte` et
  `products/+page.svelte` ; R1-2 : décimales jugées sur le texte de `decimalOf`) et `5d5c69b35` (les huit LOW : R1-3 à R1-9 ;
  R1-10 laissé, Q3). Section « Revue de code P1 et remédiation » de la fiche. Captures neuves : 4 (refaite), 9 et 10 ; journaux
  sous `$GATE_LOGS/e64/rem/`.
- Fiche : `_bmad-output/implementation-artifacts/64-1-corrections-0201.md`, Dev Agent Record et Change Log compris.
- Rapport de la P1 : `$GATE_LOGS/e64/64-1-revue-p1.md`, findings R1-1 à R1-10.
- Lis le `CLAUDE.md` du worktree : *Review Iteration Rule*, *Recette visuelle*, *Le prompt d'une passe doit NOMMER le manuel*,
  *Versions de l'API*, *Inventorier les sites NON RÉSOLUS*, *Propagation post-patch*, *Recompter*, *Priorités des défauts* et la
  règle des LOW du 2026-11-27.
- **Décisions du Project Lead du 2026-11-27, à ne pas rediscuter** :
  - Q1 : le mode se règle par l'Administrateur seul ;
  - Q2 : pas une rupture du contrat de l'API, release `c`, ⚠️ en tête des notes ;
  - #1178 réduite au bug signalé : la clé unique, le chevauchement, la collation et la concurrence vont à #1763.

## Interdits

- **Le seul fichier que tu écris** : ton rapport, par l'outil d'écriture.
- **Commandes interdites** :
  - `scripts/*`, dont `prepare-release.sh`, `test-fast.sh`, `gate-ci.sh`, `mem-guard.sh`, `recette-montee.sh` et
    `install-hooks.sh` ;
  - `docker`, `mariadb`, `sqlx`, `curl`, `wget` ;
  - `git commit|checkout|stash|restore|reset|push|worktree`, et `gh` en écriture ;
  - `sed -i`, `perl -pi`, `tee`, `>`, `>>` vers le dépôt ;
  - `pkill`, `killall`, et `kill` par motif.
- Rien sous `~/kesh/`, rien hors du worktree, de `/tmp` et de `$GATE_LOGS/e64/`.
- **Permis** :
  - `git log|show|diff|grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, et `gh … view|list` ;
  - `pdftotext … -` en tube ;
  - le manuel régénéré dans une **copie** de `docs/manual` placée sous `/tmp/claude-1000/` (`make admin user`) ;
  - Vitest d'un fichier, sous `flock $GATE_LOGS/.gate-front.lock` ;
  - le Rust ciblé, sous `flock /tmp/station-heavy.lock flock $GATE_LOGS/.gate.lock`, sur la base `kesh_e641`. Jamais `GRANT` ni
    `REVOKE`.
- Une mutation se joue sur une copie sauvegardée et se restaure par copie, puis `touch` ; vérifie ensuite que
  `git status --porcelain` est vide.

Rapport : **`$GATE_LOGS/e64/64-1-revue-p2.md`**.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Remédiation de R1-1 (MEDIUM)** — deux taux actifs de même valeur cassaient le `<select>` « TVA % ».
   - **Si la story l'a corrigé** :
     - le dédoublonnage vaut-il aux deux sites (`InvoiceForm.svelte`, chaque `(v)` ; `products/+page.svelte`, chaque
       `(opt.value)`) ?
     - reste-t-il un autre `{#each … (clé)}` sur une valeur de taux ? Inventaire clos :
       `grep -rn "#each .* as .* (" frontend/src --include=*.svelte`, puis filtre des clés qui ne sont pas un `id` ;
     - un test rougit-il quand on retire le dédoublonnage (mutation jouée) ?
     - deux taux de même valeur restent-ils distincts là où il le faut (rapport TVA, désactivés) ?
   - **Si elle a ouvert une issue** : l'issue existe-t-elle (`gh issue view`), avec sa priorité, sa story de remédiation et son
     propriétaire ? Le Change Log le dit-il ?
2. **Remédiation de R1-2 (MEDIUM)** — validateur inexact.
   - La validation porte-t-elle sur le **texte** envoyé au serveur ?
   - Sonde aux cas suivants : `8.12000000001`, `8.100000000001`, `1e-12`, `1e-7`, `8.125`, `8.105`, `0`, `0.29`, `8.1`, `99.99`,
     `100`, `100.01`, `-0`, `-1`, `1e21`, `NaN`, `Infinity`, `null`, `''`.
   - Chaque refus annoncé par AC3 empêche-t-il **à la fois** `updateVatRate` et `createVatRate` ?
   - Le miroir de `validate_rate` (`vat.rs:267-279`, `limits.rs:31-33`, `scale()` brut) est-il exact ?
   - La valeur d'un `<input type="number">` lié est un nombre, ou `null` : le champ peut-il encore rendre une chaîne non vide
     (`openChange`, `resetForm`) ?
3. **LOW de la P1** (R1-3 à R1-10) : pour chacun, corrigé dans la story ou ouvert en issue (règle des LOW du 2026-11-27) ;
   vérifie au code ou à l'issue.
   - Les compteurs du bloc `admin_routes` se **recomptent depuis la source** : `lib.rs`, entre les marqueurs, et
     `ADMIN_COUPLES` (attendu : 56 à la tête de la P1).
4. **Chaque AC contre le code et les tests**, à la nouvelle tête :
   - AC1 à AC3 : `vat-rates-page.test.ts`, `vat-rates.validate.test.ts` ;
   - AC4, AC5, AC5-bis : `invoice-edit-page.test.ts` ;
   - AC6 : `profile_e2e.rs`, `admin_pat_denied_e2e.rs` ;
   - AC7 : `user-menu-mode.spec.ts` ;
   - AC8 : manuels et guide ;
   - AC9 : notes de version de la fiche.
5. **#1758 et #1712 de bout en bout, sans t'appuyer sur la P1.**
   - Garde de rôle et de statut de la page `edit`, pour tout statut et tout rôle ; contournement de la page par l'API
     (`comptable_routes`, `invoices.rs` sous `FOR UPDATE`).
   - Inventaire clos des écritures de `onboarding_state.ui_mode`.
   - Codes rendus à une clé `read` et à une clé `read-write`, selon le rôle de son créateur ; ordre des couches
     `require_not_pat` / `require_admin_role`.
   - Menu utilisateur.
6. **Le manuel, PDF compris.** Régénère `admin-manual` et `user-manual` dans la copie, puis aplatis :
   `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`.
   - Toute phrase sur le mode d'interface, la bascule, les rôles, les opérations fermées aux clés, la modification d'une facture
     et les taux de TVA est-elle vraie au code ?
   - Vérifie aussi `docs/api-external.md` (§ 4 `:82`, le ⚠️ de `:86`, `:227`). Seules les sources françaises existent.
7. **Recette visuelle.** Si la remédiation touche un contrôle visible (formulaire de facture, catalogue, page des taux), exige
   et **regarde** les captures neuves sous `$GATE_LOGS/e64/captures/`, la capture 4 refaite comprise (la colonne « TVA % »
   doit montrer son taux).
8. **Les quatre catalogues i18n et `sitesTotal`**, recomptés depuis la source si des messages changent ;
   `npm run lint-i18n-ownership` n'est pas à lancer (script) : lis `i18n-keys.test.ts` et lance-le seul.
9. **Aucune migration** (`git diff --name-only 0e99cd2db HEAD -- crates/kesh-db/migrations`) ; aucun mot-clé de fermeture dans
   `git log origin/main..HEAD --format=%B` ; aucun chemin de la station dans le versionné.
10. **Propagation** : greppe sur tout le dépôt chaque symptôme corrigé par la remédiation — code, tests, i18n, manuels, fiche.

## Rapport

- **Les findings**, classés CRITICAL, HIGH, MEDIUM ou LOW, chacun avec sa preuve : ligne lue citée, commande et sortie, ou
  mutation.
- Tout finding CRITICAL ou HIGH qui affirme une **absence** se vérifie d'abord par `grep -nF`.
- **La liste des axes exercés et non exercés**, avec la raison de chaque non-exercice. Un « 0 finding » sans cette liste ne
  compte pas comme une passe.
- **Le décompte, recompté** depuis ton rapport.
