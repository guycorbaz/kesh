# Prompt — revue de code P1 de la Story 43-3, trois lentilles (B, E, A)

Tu relis la **Story 43-3** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB), release **0.19.0** : **l'écran
du règlement groupé** (#823), à la manière de bexio — sur une ligne du relevé, *Régler plusieurs factures…* ; recherche,
factures ajoutées une à une, total imputé, écart et son sens, sélecteur du compte de l'écart (prérempli), arrondi dit
comme une information, refus nommés ; dialogue d'annulation d'un groupe ; côté fournisseur (D6). Le modèle (43-1a,
43-1b) et l'API (43-2) sont **hors périmètre** : ils sont fusionnés dans la branche (commit `d20095425`) et relus ailleurs.

Worktree `/home/gcorbaz/devel/kesh-43-3`, branche `story/43-3-reglement-groupe-ecran`. Le diff à relire, **aplati**, sans
le travail de la 43-2 :
`git -C /home/gcorbaz/devel/kesh-43-3 diff d20095425 HEAD -- frontend docs/manual crates/kesh-i18n`
(`d20095425` = la tête de la 43-2 fusionnée par `a179b687d` ; le diff contient donc la 43-3 et les raccords de la
fusion). Langue du rapport : français. Passe **complète** (première passe), contexte frais, modèle **Sonnet** ou
**Opus** (jamais Haiku pour une P1).

## Ce qu'il faut lire d'abord

- La fiche `_bmad-output/implementation-artifacts/43-3-reglement-groupe-ecran.md` en entier (D1 à D6, AC1 à AC8), son
  Dev Agent Record, et le registre `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e433.md` (C-e433-1 à 11).
- La règle de la différence : `/home/gcorbaz/devel/kesh-gate-logs/e43/decision-difference.md` — **son tableau des six cas
  fait foi** ; le fichier d'épique `_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md` (C-43-14 à
  C-43-26).
- Le contrat réel de l'API : `crates/kesh-api/src/routes/reconciliation_group.rs` (réponses, `group_refusal_to_failed_proposal`)
  et la fiche `43-2-reglement-groupe-api.md`.
- Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
  `/home/gcorbaz/devel/kesh-gate-logs/e433/` : `front-complet-review.log`, `test-fast-review.log`, `gate-ci-review.log`,
  `e2e-complet-review.log`, `e2e-group-4.log`, `fusion-432-releve-*.log`, `rouges.log`, `mut/*.log` (le script
  `mut/mutations.py` est à **lire**, jamais à exécuter).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Les propriétés centrales

1. **Les grandeurs en entiers** (`group-settlement.ts`) : `d = |montant| − Σ amountDueRaw`, `e` au centime (demi-centime
   en s'éloignant de zéro), `r = d − e` ; aucun `Number` sur un montant ; plafond sur `|e|` seulement ; arrondi
   automatique quand `e = 0` ; compte d'arrondi requis si `r ≠ 0` ; les six cas du tableau.
2. **Le compte de l'écart** : libellé et préremplissage par famille (C-43-26), préremplissage seulement d'un compte
   offert (actif, imputable, hors banque, **hors devise** — Epic 37), choix gardé (C-e433-3), obligation dès `e ≠ 0`.
3. **Les refus** (D5) : `RETIRER_SUR`, `FERMER_SUR`, refus du compte sous le sélecteur (une phrase par cause, dont
   `ACCOUNT_IN_FOREIGN_CURRENCY`), cas de configuration (`bankAccountId`) hors du sélecteur, montants relus (C-e433-4).
4. **Les gardes** (D4, `modal-guards.ts`) : session, réponse dépassée ignorée, fermeture pendant l'envoi.
5. **La garde « déjà comptabilisée »** : geste `group`, confirmation avant la modale.
6. **L'annulation d'un groupe** : N factures, écart et compte tels que rendus, refus par facture nommé.
7. **Ce qui doit rester vrai** : `LinkSettlementModal` non touché (#1235) ; les autres gestes et leurs tests inchangés ;
   `failedProposalLabel` sans discriminant rend les libellés d'avant ; garde i18n et garde des sélecteurs en devise
   (`fx-account-filter-inventory.test.ts`) vertes.

## Les trois lentilles

- **B (Blind Hunter)** — le diff seul, sans la fiche : défauts de code Svelte/TS, effets réactifs (`$effect`, `untrack`),
  promesses non attendues, états incohérents, textes en dur, accessibilité (nom des boutons, rôles `alert`).
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e433/review-p1-B.md`.
- **E (Edge Case Hunter)** — chaque branche et chaque borne : 0, 1, 2 et 50 factures ; écart ±0.005 (`|d| = 49`, `50`),
  plafond 0.00 et réglage lu de la réponse ; signe qui change ; sélecteur qui disparaît et revient ; réglages
  illisibles ; recherche qui échoue (404, 409, 412) ; réponse hors d'ordre ; refus de la transaction pendant
  l'envoi ; facture refusée sans `invoiceNumber` ; débit (D6) ; compte en devise. Rapport : `…/review-p1-E.md`.
- **A (Acceptance Auditor)** — AC1 à AC8 et les décisions D1 à D6 contre le code et les tests ; tests et mutations
  annoncés par la fiche (§ *Tests*) présents et justes ; **manuel contre code, PDF compris** :
  `docs/manual/fr/user-manual.tex`, sections `sec:reglement-groupe`, `sec:annuler-rapprochement`, la confirmation
  « déjà comptabilisée » (`sec:deja-comptabilisee`), et les passages sur le lot pain.001 (« Un lot de paiement en cours
  sur ce compte », l'encadré « Dépense déjà réglée », « Ce qui ne se rattache pas ») — un lot pain.001 reste à rejeter,
  seules les factures ouvertes hors lot se règlent en groupe ; le **PDF jetable**
  `/home/gcorbaz/devel/kesh-gate-logs/e433/manuel/user-manual.pdf` (compilé hors du dépôt ; le PDF du dépôt ne change
  pas dans une story) se contrôle aplati ; **captures** `/home/gcorbaz/devel/kesh-gate-logs/e433/captures/*.png`
  (thème clair) contre AC8 — chaque état demandé a-t-il sa capture, et montre-t-elle ce qu'elle dit ?
  Rapport : `…/review-p1-A.md`.

Chaque rapport : findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec le fichier et la ligne lus (sortie de
`grep -nF` copiée pour toute affirmation de présence ou d'absence), et **la liste des axes exercés et non exercés**.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, ni `pkill`, ni `killall`. Aucun `git` qui écrit (`checkout`, `switch`, `restore`, `stash`, `commit`,
  `merge`).
- **Rien qui compile ou exécute** : ni `cargo`, `npm`, `npx`, `node`, `python3`, `perl`, `bash -c`, `jq`, `docker`,
  `mariadb`, `curl`, `make`, `xelatex`, `claude`, `sed -i` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
  `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `e2e-e433.sh` et
  `mut/mutations.py`). **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et l'outil de
  lecture d'images pour les captures. Contrôle du PDF sous la seule forme
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
