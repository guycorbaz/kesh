# Validation P3 — fiche v0172-annulees (Sonnet, passe complète, lecture seule)

Tu es la lentille de validation P3 de la fiche BMAD **v0172-annulees** de Kesh (release 0.17.2, issue #868 : repasser en
brouillon une facture fournisseur annulée). Modèle : **Sonnet**. Contexte frais. La P1 (Sonnet) a rendu 0/0/7/8, la P2 (Opus) 0/1/4/8 ; la fiche a été remédiée après chacune (Change Log, entrées 0.2 et 0.3) : tu ne pars **pas** de leurs rapports, tu relis la fiche entière. Passe complète : un HIGH est sorti en P2. Travaille en français soigné.

## À lire, en entier

1. La fiche : `/home/gcorbaz/devel/kesh-0172a/_bmad-output/implementation-artifacts/v0172-annulees.md`.
2. Le registre : `/home/gcorbaz/devel/kesh-0172a/_bmad-output/implementation-artifacts/v0172-choix-autonomes.md`
   (section v0172-annulees, C-v0172a-1 à 23).
3. Le mandat de la fiche : `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0172-annulees-fiche.md` (la décision du Project
   Lead du 2026-11-04, à la lettre) et `gh issue view 868 --json title,body` (lecture seule).
4. `/home/gcorbaz/devel/kesh-0172a/CLAUDE.md` : *Review Iteration Rule*, *Migration breaking policy*, *Issue Tracking
   Rule*, *Inventorier les sites non résolus*.
5. Le code, au commit `d7ec2ea2` (worktree `/home/gcorbaz/devel/kesh-0172a`, à lire par `Read`, `git grep`, `git show`) :
   `crates/kesh-db/src/repositories/supplier_invoices.rs` (`unvalidate_in_tx`, `delete_draft_in_tx`, `cancel_in_tx`,
   `cancel_settlement_in_tx`), `journal_entries.rs` (`delete_in_tx`, `modification_guard`, `reverse_in_tx_inner` étape 5),
   `letterings.rs` (`dissolve_group_in_tx`, `Origin`, `Mode`, `discover_supplier_invoice_document`),
   `reconciliation_cancel.rs`, `crates/kesh-db/src/errors.rs`, `crates/kesh-api/src/errors.rs`,
   `crates/kesh-api/src/routes/supplier_invoices.rs`, `crates/kesh-db/src/repositories/invoices.rs` (`unvalidate`),
   `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`,
   `frontend/src/lib/features/supplier-invoices/supplier-invoice-unvalidate.ts`, les tests cités par la fiche.
6. **Le manuel** : `docs/manual/fr/user-manual.tex` (§ *Le brouillon*, *Dévalider une facture validée*, *Annuler une
   facture fournisseur*, numérotation) **et son PDF** `docs/manual/fr/user-manual.pdf`, `docs/manual/fr/admin-manual.tex`
   (§ *Conformité OLICo Art. 9*) **et son PDF**. Contrôle le PDF aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o "…"`.

⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`.

## Ce que tu vérifies (axes)

- **A0 — Prémisses au code, et documents neufs.** `docs/api-external.md` (`:560-575`, `:869`, `:963`) et
  `docs/i18n-glossaire.md:96` sont à lire comme le manuel.
- **A1 — Prémisses au code.** Chaque affirmation du § *Ce que le code dit* (numéros de ligne, comportements, ordres de
  refus, verrous) est-elle vraie à `d7ec2ea2` ? Copie la ligne lue (sortie `git grep -nF` ou extrait `Read`) pour toute
  affirmation que tu contestes **ou** que tu confirmes comme critique.
- **A2 — Faisabilité du geste.** L'ordre prescrit (tâche 3 : verrous, dissolution, `UPDATE`, `delete_in_tx` ×2) aboutit-il
  réellement ? En particulier : l'étape 3-bis de `delete_in_tx` pour l'achat une fois la contre-passation supprimée dans
  la même transaction ; la FK `reverses_entry_id` ; la contrainte `chk_supplier_invoices_entry_iff_not_draft` et toute
  autre contrainte `CHECK` de `supplier_invoices` sur un brouillon (lire les migrations) ; le mode `System` de
  `dissolve_group_in_tx` (exercice tenu, couverture d'une ligne) ; les groupes `reversal` réellement créés par
  l'annulation (ligne 2000, impôt préalable ; facture annulée alors que payée : dissolution du groupe `document` puis
  lettrage).
- **A3 — Verrous et instantané.** « Les verrous d'abord » (leçon 25-3-b) : une lecture ordinaire prescrite avant un
  verrou ? Cycles nouveaux avec l'annulation, le paiement, le lettrage, la clôture ? Le rejeu de la route les couvre-t-il ?
- **A4 — Inventaires clos.** (a) sites qui écrivent ou lisent `cancelled` (fournisseur) — rejoue le grep, cherche un site
  manquant (rapports `kesh-report`, MCP, export, sauvegarde, réconciliation, contrôles de clôture, écrans) ;
  (b) `SupplierUnvalidationBlocker` et refus de la dévalidation — chaque refus dit-il quoi faire d'abord ? un refus
  atteignable omis ? un refus déclaré inatteignable qui l'est ?
- **A5 — Le paiement détaché (AC-6, C-v0172a-5).** Le défaut décrit (gel levé par le passage en brouillon) est-il réel ?
  La solution le ferme-t-elle, y compris après `delete_draft` (facture supprimée) ? Le chemin « contre-passer le
  paiement » est-il réellement disponible (fiche de l'écriture, `/reverse`, réconciliation) ?
- **A6 — Point 2 du mandat.** Le parcours payée → rapprochement annulé → validée → brouillon → supprimée existe-t-il tel
  que la fiche le décrit (`reconciliation_cancel` → `cancel_settlement_in_tx`), sans changer le refus ?
- **A7 — Point 4 du mandat (clientes).** L'écart est-il bien établi, et la conclusion « rien ne change » justifiée ?
- **A8 — AC décidables et tests.** Chaque AC a-t-il un test nommé qui rougirait si l'AC était faux ? Les mutations
  nommées (garde d'exercice, suppression de la contre-passation, trace d'audit, et les autres) sont-elles tuées par les
  tests indiqués ? M12 est-elle vraiment équivalente ? Tests existants à réécrire bien identifiés (Rust, Vitest, E2E) ?
- **A9 — Écran, i18n, quatre locales.** Clés neuves et modifiées, replis = FTL fr-CH, arguments plats, `isStaleState`,
  `data-testid` ; captures prescrites.
- **A10 — Manuel et piste d'audit.** Le § *Manuel* corrige-t-il **toutes** les phrases du `.tex` **et du PDF** qui
  deviendraient fausses (greppe « ne supprime aucune facture annulée », « ne se dévalide pas », « Dévalider »,
  « contre-passation », « trou ») ? La lecture CO 957a / 958f / Olico est-elle exacte et prudente ? Le manuel
  d'administration est-il couvert ?
- **A11 — Migrations et production.** « Aucune migration » est-il vrai pour tout ce que la fiche prescrit ? Retour à la
  0.17.1 sans restauration : exact ?
- **A12 — Registre.** Chaque choix C-v0172a-k a contexte, retenu, écartées, réversibilité ; aucune décision de la fiche
  sans choix au registre ; cohérence fiche ↔ registre (numéros, codes d'erreur, noms de clés).

- **A13 — La remédiation de la P1.** Le diff de la remédiation se lit par `git diff 6238701a HEAD --
  _bmad-output/implementation-artifacts/` (lecture seule). Pour chacun des quinze findings V1-1 à V1-15 (rapport
  `/home/gcorbaz/devel/kesh-gate-logs/v0172a-validate-p1.md`, à lire **après** ta propre lecture de la fiche) : le
  correctif ferme-t-il le défaut ? a-t-il introduit un défaut neuf — en particulier : la lecture verrouillante de la
  contre-passation (tâche 3.3) et l'ordre lecture d'audit → refus ; la précédence par écriture et les paires P1 à P6
  (chacune montable ? chacune fait-elle jouer les deux refus ? P4 tue-t-elle M12 ?) ; la séquence `paid` sans boîte
  (C-v0172a-21 : la protection par la version tient-elle, statut lu avant la version ?) ; la liste close des clés
  (AC-15 : décomptes 5 et 15 recomptés, le grep de contrôle est-il juste et décidable dans les quatre locales ?) ; le
  repli `format!` (AC-8, T7b) ; les lignes de `docs/api-external.md` (AC-16) ; le tableau des tests à réécrire (liste
  vraiment close ?). Le symptôme corrigé a-t-il été propagé partout dans la fiche et le registre (greppe les valeurs :
  « équivalente », « Olico art. 3 », « rangs 6 à 10 », « (Lot 4) », « Dévalider », anciens numéros de ligne) ?

- **A14 — La remédiation de la P2.** Diff : `git diff 0bded2c5 HEAD -- _bmad-output/implementation-artifacts/` (lecture
  seule, en flux). Rapport P2 : `/home/gcorbaz/devel/kesh-gate-logs/v0172a-validate-p2.md`, à lire **après** ta propre
  lecture. Pour chacun des treize findings V2-1 à V2-13 : fermé ? défaut neuf ? En particulier :
  - le contrôle de vocabulaire d'AC-15 (liste `K` de 17 clés, racine par langue) : **rejoue-le à `d7ec2ea2`** par
    `git show d7ec2ea2:crates/kesh-i18n/locales/<loc>/messages.ftl | /usr/bin/grep -E "$K" | …` (en flux, sans fichier) et
    confirme la sonde annoncée (`clés=17 verbe=13` dans chaque locale) ; les racines visent-elles exactement *dévalider*
    (aucune clé de la liste qui doive garder le mot, aucune forme du verbe manquée — de « aufgehoben », « Heben … auf »,
    it « annullarne la convalida ») ? La liste `K` est-elle close (une clé fournisseur portant le verbe hors de `K`) ?
  - la liste des 14 replis (rejoue le `git grep` cité) et des tests à réécrire ;
  - l'inventaire des manuels (46 lignes de l'utilisateur, 10 de l'administration) : rien d'oublié, PDF compris ;
  - le contrôle DRY sur `'$.settlementJournalEntryId'` et le fragment SQL partagé (AC-6) : réalisable, et l'ordre
    « lectures ordinaires après le dernier verrou » tenu ?
  - V2-5 : l'effet par période est-il bien écrit, testé (T1) et justifié (C-v0172a-23) ; l'argument « le verrou de
    période suffit » tient-il ?
  - la propagation des valeurs corrigées (« aucun solde », « quinze », « 15 », « IN ('open','cancelled') »,
    « lettering_key IS NULL », « Freigabe aufheben ») dans la fiche et le registre, entrées passées du Change Log exceptées.

## Sévérités

CRITICAL (le geste corromprait les livres ou ne peut pas aboutir), HIGH (AC faux ou intestable, défaut comptable non
traité), MEDIUM (prémisse fausse, inventaire incomplet, test qui ne tuerait pas sa mutation, manuel faux), LOW (rédaction).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout`/`switch`
(sous aucune forme), `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `rebase` ; aucune commande `gh` qui
écrit (`gh issue view` en lecture seulement) ; pas de `claude`, `pkill`, `killall` ; pas de `sed -i`, `perl -pi`, `tee`,
redirection `>` vers le dépôt. ⛔ **Aucune redirection** (`>`, `>>`, `2>`, `tee`, `-o`), **aucun fichier temporaire**, y compris dans `/tmp`, le
scratchpad de la session ou tout répertoire de travail : toute sortie se lit en flux (`git show … | …`). Incident de la
P2 : un `git show … > …/scratchpad/lett.rs` a écrit hors du dépôt — c'est interdit. **Tu n'écris qu'un seul fichier** :
ton rapport, par l'outil d'écriture, jamais par le shell.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0172a-validate-p3.md` (chemin absolu), en français :

1. En-tête : modèle, date, commit lu.
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW` (recompté depuis ta liste).
3. Findings numérotés `V3-n` : sévérité, axe, citation de la fiche, preuve au code (ligne lue copiée), correctif proposé.
4. **Axes exercés et axes non exercés** (A0 à A14, un par un, avec ce que tu as réellement lu ou greppé). Un « 0 finding »
   sans cette liste ne compte pas comme passe.
