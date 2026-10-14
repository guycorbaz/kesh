# Prompt — validation P2 de la spec, Story 29-5 (à score égal, la facture échue la première passe en tête)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; **P2 : Opus**
pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-29-5`, branche `story/29-5` (base `fcd2ae75`, `main` après le kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-5-departage-exigibilite.md`. **La remédiation de la P1 est le commit
`fdf3c949`** : `git show fdf3c949` (fiche et registre). Rapports de la P1, à lire pour savoir ce qui a été signalé :
`/home/gcorbaz/devel/kesh-gate-logs/29-5-validate-p1-R.md` (0/0/4/4) et `…-p1-F.md` (0/0/2/5) ; ce que la remédiation
en a fait : entrée « Validation P1 » du Change Log de la fiche.
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-5*. Issue #668 **et son commentaire de
rectification** : `gh api repos/guycorbaz/kesh/issues/668` et `gh api repos/guycorbaz/kesh/issues/668/comments`.
Voisines **non mergées**, en lecture seule : `/home/gcorbaz/devel/kesh-29-2/_bmad-output/implementation-artifacts/29-2-paiement-tardif.md`
(AC1 : ancre ; AC5 : `Vec` ordonné, M15 ; registre `epic-29-choix-autonomes-29-2.md`, C-29-2-7, C-29-2-11, C-29-2-13)
— ⚠️ son agent de développement travaille dans ce worktree : lis, n'y touche rien, et si du code de l'ancre y existe
déjà (non commité ou commité), dis quelle forme il a ; `/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`
(AC1 `order_candidates`, AC8 : entrée du CHANGELOG et phrase de `api-external.md` ; C-29-3-12). Mergée : #548
(Story 28-7, C-28-7-10).
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-5.md` (C-29-5-1 à 12 ; 11 et 12 neufs,
9 amendé).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § *Un
appariement automatique propose, il ne crée jamais*) ; `docs/agents-orchestration.md`.

## Le texte de l'art. 87 CO (repris de la P1)

> **Art. 87** — ¹ Lorsqu'il n'existe pas de déclaration valable, ou que la quittance ne porte aucune imputation, le
> paiement s'impute sur la dette exigible ; si plusieurs dettes sont exigibles, sur celle qui a donné lieu aux
> premières poursuites contre le débiteur ; s'il n'y a pas eu de poursuites, sur la dette échue la première.
> ² Si plusieurs dettes sont échues en même temps, l'imputation se fait proportionnellement.
> ³ Si aucune des dettes n'est échue, le paiement s'impute sur celle qui présente le moins de garanties pour le
> créancier.

⚠️ Le texte remis aux lentilles de la P1 n'a pas été versionné ; celui-ci est reproduit pour la P2. Si tu peux lire
la source (Fedlex, RS 220, art. 75, 86, 87) en lecture seule, confronte-le **et** la fiche à la source et dis l'écart
éventuel ; sinon, dis-le.

## Axe obligatoire — relire la remédiation P1 (`git show fdf3c949`)

La sévérité se déplace vers ce qu'on vient d'écrire. Pour **chaque** finding de la P1 (R1-R8, F1-F7), dis s'il est
**fermé**, **fermé à moitié** ou **rouvert sous une autre forme**, et cherche ce que le correctif a **introduit** :

- **F1 — la forme macro** (AC1, C-29-5-11) : `concat!("CAST(GREATEST(", invoice_exigibility_date_sql!(), ", …)) AS DATE)")`
  est-il du Rust valide dans un corps de `macro_rules!` (un appel de macro défini dans le même module, rendant un
  littéral, dans un `concat!`) ? Le précédent cité (`line_ttc_sql!`, `repositories/invoices.rs:186-216`) l'établit-il
  vraiment ? L'imbrication `CAST(GREATEST(CAST(GREATEST(…) AS DATE), …) AS DATE)` est-elle équivalente à l'ancre de la
  29-2 pour toute ligne (y compris `NULL`) ? La promesse « aucun test de la 29-2 ne change » est-elle tenable, et le
  repli (forme mergée différente) assez précis pour ne pas devenir une seconde écriture silencieuse ?
- **F2 / R8 — les deux montages à 51 factures** (AC4) : recalcule à la main, ligne par ligne, l'ordre rendu sous
  l'ordre correct et sous **chacune** des mutations M8, M9, M12 ; les dates sont-elles dans la fenêtre au sol **et**
  avec la 29-2 (`F = D + 1` admis ? `F = D − 30` ?) ; les assertions « écrites en dur » le sont-elles sans passer par
  la fonction testée ; le décompte (1 + 1 + 48 gardées, 49 + 1 gardées) est-il exact ?
- **R1 — la sixième ligne et M3** : la ligne 6 du tableau est-elle exacte, et M3 est-il désormais tué — et seulement
  par les tests que la colonne nomme ? Les autres colonnes « Tuée par » ont-elles bougé à juste titre (renumérotation
  des cas d'AC5 : 6 neuf, 7, 8, 9) ?
- **R2 — la condition de montant** : le texte prescrit du manuel, du CHANGELOG et la prémisse disent-ils ce que fait
  `matching.rs:80` (et `amount_score`, `reference_score`, la demi-référence 0,5 sur un préfixe) — ni plus, ni moins ?
  Le troisième sous-cas de l'AC5 cas 8 a-t-il les scores annoncés (0,50 contre 0,60) ?
- **R3 — les `Vec` de la 29-2** : la lecture de son AC5 (montants distincts par cas, cas inter-factures) est-elle
  juste ? L'exception « assertion réécrite et déclarée » contredit-elle une autre phrase de la fiche 29-5 ou de la
  29-2 (« sans modification d'assertion ») ?
- **R4 / R6 — les phrases de la 29-3** : les deux parenthèses de remplacement prescrites (AC7) sont-elles vraies au
  code prescrit, et la phrase de la 29-3 reste-t-elle grammaticale et juste une fois la parenthèse changée ? La
  correction de l'entrée du CHANGELOG de la 29-3 respecte-t-elle « une entrée publiée ne se réécrit pas » ?
- **LOW** (F3-F7, R5, R7) : chaque correction est-elle exacte, et le grep de § *Propagation* élargi rend-il bien
  les sites qu'il prétend viser ?

## Axes prioritaires (communs aux deux lentilles)

1. **Prémisses** (§ *Les prémisses*, dont les deux rangées récrites) : revérifie au code de `fcd2ae75`. Une prémisse
   fausse ou ignorée est au moins **MEDIUM**.
2. **Le critère légal**, au texte ci-dessus : les quatre points, la convention « exigible = échue » pour une facture
   sans terme (C-29-5-12) — est-elle honnêtement écrite comme convention, et sa conséquence (al. 3) exacte ? Une
   règle contraire au texte est **HIGH**.
3. **L'équivalence « échues d'abord ⇔ `X` croissante »** et la totalité de la clé `(X, F, id)` : un contre-exemple
   est **HIGH**.
4. **La troncature** (AC3, F3 : la victime change) : un cas où le `LIMIT 50` écarterait encore, parmi des scores
   égaux, la facture que l'art. 87 désigne est **HIGH** ; ce qui reste non résolu est-il écrit ?
5. **Périmètre minimal et fermeture de #668** : AC nécessaires, attendu révisé couvert par un test rouge au sol,
   quatre modules (recompte).
6. **Inventaire clos des sites** (Dev Notes (a), (b), (c)) : refais et élargis les greps ; un site ou un test ignoré
   dont le sens change est au moins **HIGH**.
7. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` § *Algorithme de matching*
   (l. 2027-2052) — ce qu'il dit aujourd'hui, et ce que l'AC7 prescrit (désignation à montant égal, al. 3, ordre
   entre non échues, `E < F`, clients différents, recours par la fiche puis *Rejeter*). Contrôle aussi le **PDF**,
   aplati et ligatures normalisées :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g'`. Même
   exigence pour les phrases prescrites de `docs/api-external.md` et du CHANGELOG. Une promesse que le code ne
   tiendrait pas est au moins **MEDIUM**.
8. **Recomptes** depuis la source : AC (9), tâches (T0-T6), mutations (M1-M13), choix (12), modules (4), lignes
   d'exemple (6), cas d'AC5 (9, dont 8 neufs), tests neufs de `reconciliation_repository.rs` (3), et les nombres de
   l'entrée « Validation P1 » du Change Log.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #668, de sa rectification et du § *29-5* de l'epic couverte par un AC testable, chaque AC par une
  tâche et par un test **qui rougirait** ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`,
  `grep -nF`), y compris ceux que la remédiation a ajoutés (`accounts.rs:20-34`, `invoices.rs:50-65`, `:186-216`,
  `matching.rs:109-119`, `:375`, `:386`, `reconciliation.rs:208-213`, `email_template_defaults.rs:68`, `:96`,
  `CHANGELOG.md:110`) ; les zones partagées avec la 29-2 et la 29-3 confrontées à leurs fiches.
- **F — Faisabilité et exactitude de la règle (adversaire de la conception)** : la macro et sa composition (F1
  ci-dessus) ; l'expression en MariaDB 10.11 dans l'`ORDER BY` à côté du `HAVING` et des jointures (et de `rmd`) ;
  le tri `NULL` (M9) ; la fonction Rust et le piège `Option` (M3) ; les montages à 51 factures et leurs mutations ;
  les cas de l'AC5 (scores, ordre d'entrée, stabilité de `sort_by` pour M13) ; les mutations M1-M13 toutes
  **tuables** par les tests nommés, et en manque-t-il une évidente (par exemple côté SQL, `i.id` retiré de
  l'`ORDER BY`) ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dis s'il vient de la
**remédiation P1** (`fdf3c949`) ou de la conception d'origine. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. Le **bilan des findings de la P1** (fermé / à moitié / rouvert). ⛔ **La liste des axes exercés
ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/29-5-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-5-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach`
compris)/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `rustc`, `npm`, `npx`,
`node`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui écrit),
`docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy)
ne se lit ni ne se cite. Les worktrees `kesh-29-2` et `kesh-29-3` se lisent, ne se modifient pas — et l'on n'y lance
aucune commande `git` autre que `log`/`show`/`diff`/`status`. Autorisés : lecture, `grep`, `sed -n`, `git
log`/`show`/`diff`/`blame`/`status`, `gh api` en lecture, `pdftotext` vers la sortie standard, lecture web.
