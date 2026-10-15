# Prompt — validation P2 de la spec, Story 30-2 (des documents à traiter à la facture fournisseur)

*Versionné le 2026-10-15. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule stricte. Rotation
(D6, rétro 25) : P1 Sonnet → **P2 Opus** pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`, tête `079bba30` (remédiation de la P1). Fiche :
`_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` — une fiche, **trois** stories : 30-2a
(AC1-AC9, AC6-bis, AC9-bis), 30-2b (AC10-AC17, AC12-bis), 30-2c (AC18-AC24). Registre de la story :
`epic-30-choix-autonomes-302.md` (C-30-2-1 à 21). Epic : `_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md`
§ 30-2 et registre `epic-30-choix-autonomes.md`. Issue #685 — **conception de Guy, elle fait foi** (`gh api
repos/guycorbaz/kesh/issues/685` et `…/685/comments`, lecture seule). Rapports de la P1 :
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p1-R.md` et `…-p1-F.md` ; leur remédiation est l'entrée « Validation
P1 » du Change Log de la fiche. Règles : `CLAUDE.md` (Review Iteration Rule et ses sous-sections, Règle de splitting
préventif, Migration breaking policy P1-P8, Un appariement automatique propose, Issue Tracking Rule) ;
`docs/agents-orchestration.md`. La Story 30-1 (#686) sera développée avant : la fiche ne touche pas
`kesh-reconciliation`.

⚠️ **Motif connu** : la sévérité se déplace vers ce qu'on vient d'écrire. La P1 a ajouté ou réécrit AC2, AC3, AC4,
AC6, **AC6-bis**, AC7, AC8, AC9, **AC9-bis**, AC10, AC11, AC12, **AC12-bis**, AC13, AC14, AC16, AC17, AC18, **AC19**,
AC21, AC23, AC24, les mutations, le § E2E et le § *Gardes existantes*. Relis-les en premier, mais la passe reste
**complète** (toute la fiche).

## Axes prioritaires (communs aux deux lentilles)

1. **La remédiation de la P1 tient-elle, et n'a-t-elle rien cassé ?** Pour chaque ligne du tableau de remédiation du
   Change Log, le remède est-il dans la fiche, juste au code, et cohérent avec les autres AC ? Cherche les résidus :
   une phrase qui dit encore l'ancienne conception (lecture verrouillante de l'étape 3, longueurs partielles,
   `sourceDocument … | null` à deux états, validation « par le même chemin », `test.skip`), un décompte ou un numéro
   de ligne périmé.
2. **Inventaire et parcours de #685** : les prémisses P-a à P-j et « Ce qui existe » restent-elles vraies ? Chaque point
   du parcours (1 à 7) a-t-il un AC testable et un E2E ? Le découpage tient-il (modules par sous-story, chaque
   sous-story livrable) ?
3. **Migration et version** (P1-P8) : AC4 et AC9-bis — un seul `ALTER TABLE` dans l'ordre prescrit est-il valide en
   MariaDB 10.11 (MODIFY ×7, CHECK, ADD UNIQUE, DROP INDEX de la clé étrangère) ? `min_required` et crates (P2-bis),
   P5, P6, P7, squash. AC12-bis change le comportement de `POST /supplier-invoices` : quels tests, E2E, données de
   seed ou fixtures portent un IBAN factice ou un QR-IBAN en `creditorIban` et rougiront (cherche-les, la fiche les
   laisse au développeur) ? La conversion commune peut-elle appeler `kesh_qrbill::validation` (signatures, erreurs) ?
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` (§ import l. 1712-1741, § saisie
   `sec:saisie-facture-fournisseur` l. 1568, K5 l. 801 et 808) et `docs/manual/fr/admin-manual.tex` (l. 793, 855-880,
   1456, 1599, *Passer à la 0.13.4* l. 2012) : ce qu'ils disent aujourd'hui est-il vrai au code ? Ce que AC9, AC9-bis,
   AC17 et AC23 prescrivent d'écrire sera-t-il vrai au code prescrit (propositions de fournisseur, validation des IBAN,
   détachement, #503) ? Les phrases gardées par `textes_coherents.rs` G5 survivent-elles ? Contrôle aussi le **PDF**,
   aplati vers la **sortie standard seulement** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   (de même pour `admin-manual.pdf`) — jamais vers un fichier.
5. **Recomptes** : 27 AC, 23 tâches, 33 mutations dont trois déclarées équivalentes, neuf parcours E2E, 21 choix, les
   onze gardes du § *Gardes existantes*, chaque numéro de ligne cité — recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque AC couvert par une tâche et par un test **qui rougirait** ; chaque mutation tuée par le test nommé, et chaque
  mutation déclarée équivalente (M-a11, M-b2, M-c7) l'est-elle vraiment ? Une mutation équivalente non déclarée est un
  finding. Les E2E (§ *E2E de parcours*) : le `beforeAll` qui échoue, les copies uniques par suffixe après la fin
  logique (un PNG à QR suffixé se décode-t-il encore — `rxing`/`image` dans `crates/kesh-api/src/qr_decode.rs` ? un
  PDF suffixé reste-t-il valide pour le navigateur ?), l'attribution des fixtures, l'indépendance des tests et leur
  rejeu. AC6-bis est-il un critère vérifiable (qui, quoi, où consigné) ? AC14 : la règle d'éligibilité de
  `proposeSuppliers` est-elle décidable et testable telle qu'écrite (formes juridiques après `fold`, « tous les mots
  du plus court ») ; laisse-t-elle passer un faux rattachement plausible, ou écarte-t-elle un vrai fournisseur courant
  (« Dubarde SàRL » / « DUBARDE S.à r.l. ») ? Textes : i18n ×4, clés retirées et ajoutées, erreurs neuves, manuels,
  CHANGELOG et son bandeau, `api-external.md` (section **créée**), `README.md:44`. Écrans : rôles, états vide / erreur
  / chargement / 410, largeur étroite, clair et sombre. Règle « un appariement propose » : aucun chemin ne rattache sans
  geste humain.
- **F — Faisabilité, sécurité, verrous et naissance de la vue (adversaire de la conception)** :
  - **Verrous** : AC19 sans lecture préalable — l'`UPDATE` de `mark_completed`
    (`crates/kesh-db/src/repositories/imported_supplier_invoices.rs:177-205`) sur l'index d'unicité : quels verrous
    pose-t-il (contrôle de doublon, entrée d'index) ? Deux rattachements vers deux factures différentes sont-ils
    réellement sans attente ni cycle ? Rattachement contre complétion (`complete_import_once`, qui insère une facture
    puis écrit le lien), contre retrait (`discard_import`), contre réactivation, contre détachement (AC24, verrou du
    seul document) : écris la matrice. La traduction de `UniqueConstraintViolation` en `409` ne capte-t-elle que
    `uq_imported_supplier_invoice` (l'`UPDATE` peut-il violer une autre unicité) ? Le test déterministe de M-c10
    (facture tenue par une autre connexion, borne de 5 s) est-il montable et prouve-t-il ce qu'il prétend ?
  - **Naissance de la vue** : AC18 — la lecture du staging lié entre-t-elle vraiment dans l'instantané de
    `get_settlement_view` (`repositories/supplier_invoices.rs:1519`) sans déplacer sa naissance ni ajouter de lecture
    verrouillante ? AC21 — idem pour le détail de l'écriture (`routes/journal_entries.rs:485`). AC13 — `complete` relit
    par le GET après validation : la réponse peut-elle refléter un état postérieur (course avec un paiement) et est-ce
    acceptable ?
  - **Sécurité** : CSP finale (`img-src … blob:`, `frame-src 'self' blob:`, `object-src 'self' blob:`) : surface
    ouverte (contenu actif d'un PDF dans un cadre de même origine, `<object>` injecté) contre ce que la CSP actuelle
    permet (`crates/kesh-api/src/middleware/csp.rs:24-32`) ; type MIME toujours dérivé de l'extension ; IDOR du
    rattachement et du détachement des deux côtés ; lecture ouverte à tout rôle bornée à la société ; bornes de
    l'import intactes ; AC12-bis : la validation peut-elle être contournée (espaces, minuscules, normalisation avant
    contrôle) ?
  - **Faisabilité** : contrôle de longueur en caractères pour toutes les colonnes bornées du QR (AC2) ; extraction de
    `SupplierInvoiceForm.svelte` sans changer le comportement du formulaire de la liste ; `isQrIban` côté écran et la
    même règle côté serveur (une seule définition de « QR-IBAN » ?) ; `Option<Option<…>>` sérialisé comme décrit
    (serde : `skip_serializing_if` sur l'extérieur, `null` pour l'intérieur) ; la route `DELETE` à côté d'un `GET` et
    d'un `POST` de même chemin dans deux routeurs de rôles différents (`lib.rs:506-513`, `:868-875`) ; le registre des
    routes et `docs/api-external.md` l. 788.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. Dis pour chaque finding s'il vient de la **remédiation P1** ou de la conception d'origine. ⛔ **La
liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/`. Lecture seule stricte :
**aucun `git checkout`, `git switch`, `git checkout --detach` ni aucune autre forme** ; aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`,
**`python3`, `python`, `perl`, `ruby` et tout autre interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête
SQL**, `bash` sur un script du dépôt, `gh` en écriture. `pdftotext` vers la **sortie standard** seulement, jamais vers
un fichier. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`). Autorisés : lecture, `grep`,
`sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext … -`.
