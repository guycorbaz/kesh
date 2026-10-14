# Story 29-4b — revue de code P1 (trois lentilles B, E, A)

*Prompt versionné de la passe 1 de `bmad-code-review` de la clé `29-4b-reverification-annulations` (#651). Modèle :
**Opus**. Trois lentilles indépendantes, chacune en **contexte frais**, chacune écrit **son** rapport.*

## Contexte commun aux trois lentilles

- Dépôt : `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4b`. **Base** : `fe404b11` (`main`, la 29-4a mergée).
  **Périmètre** : le diff `git diff fe404b11..HEAD` — un seul diff aplati, pas la suite des commits. Il porte deux
  commits de code : `dba87062` (étape 0, **T6-bis** : les cinq points reportés de la revue de la 29-4a) et `6c8513a6`
  (la 29-4b proprement dite).
- Fiche : `_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` — elle porte **deux** clés ; la
  **partie b** (AC6 à AC8, AC9 part b, AC10) et la tâche **T6-bis** sont dans le périmètre ; la partie a est mergée
  (sa revue est close). Registre des choix : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md`
  (C-29-4-32 à C-29-4-37 sont ceux du développement de la 29-4b). Dev Agent Record de la fiche, § *Partie b* : ce qui a
  tourné, avec les chiffres.
- `CLAUDE.md` à la racine fait foi (*Review Iteration Rule*, *Pattern batch*, *Recompter ses propres comptes rendus*).
- Fichiers de production touchés — **code** : `crates/kesh-db/src/repositories/reconciliation_cancel.rs` (W9),
  `invoice_settlements_write.rs` (W25), `supplier_invoices.rs` (W28, W29), `bank_imports.rs` (W24, doc-comment de
  l'étape 5 et de `innodb_snapshot_isolation`), `bank_accounts.rs` (fonction neuve `primary_flag_locked_in_tx`, R-2 ;
  doc-comments du § « fenêtre de la clôture », des cycles, du module), `crates/kesh-api/src/routes/bank_accounts.rs`
  (`archive_once` relit `is_primary`, R-2 ; doc-comments R-4) ; **doc-comment seul** :
  `crates/kesh-api/src/routes/reconciliation.rs` (`closed_account_failed`). Tests :
  `crates/kesh-api/tests/{bank_accounts_closed_e2e,rejeu_interblocage_e2e,bank_account_closed_inventory,bank_accounts_e2e,audit_route_registry}.rs`.
  Documents : `docs/manual/fr/user-manual.tex` **et son PDF**, `docs/api-external.md` (l. 242, § *Compte bancaire
  clos* des lots, § *Annuler le rejet*, § *Interblocages*), `CHANGELOG.md` (§ `[0.13.4]`, `### Corrigé`).
- **Manuel** : la puce du § *Clore un compte bancaire* (paragraphe renommé « Deux précisions ») est réécrite. Le
  contrôle porte sur le **PDF** aussi, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
  (ne contient plus « hors de cette fenêtre », contient « ne passe pas sur le compte clos »). Vérifier que le manuel
  et `docs/api-external.md` disent **vrai du code** — c'est en vérifiant ce que le texte promet qu'on trouve un chemin
  d'écriture oublié.

## Interdits — lecture seule stricte

- **Aucune** commande qui écrit, compile ou exécute : ni `cargo` (aucune sous-commande), ni `npm`, ni `make`, ni
  `docker`, ni `mariadb`, ni `sqlx`, ni `python3`, ni `perl`, ni `sed -i`, ni script du dépôt — **en particulier**
  `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`.
- **Aucun** `git checkout`, `git switch`, `git stash`, `git reset`, `git commit`, sous **aucune** forme (ni `--detach`,
  ni sur un fichier). Les seules commandes `git` permises : `git diff`, `git show`, `git log`, `git grep`,
  `git blame`.
- Aucun fichier écrit **hors** de ton rapport. Permis : `Read`, `grep`, `rg`, `pdftotext … -` vers la sortie standard.
- Toute affirmation de **présence** ou d'**absence** d'un code s'appuie sur une sortie `grep -nF` **copiée** dans le
  rapport.

## Rapport exigé (chaque lentille)

Fichier : `/home/gcorbaz/devel/kesh-gate-logs/29-4b-review-p1-<B|E|A>.md` (jamais sous `target/`). Contenu :

1. **Axes exercés** et **axes non exercés**, listés — un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `<lentille>-n`, chacun : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), fichier et ligne,
   constat, preuve (sortie copiée), remède proposé, et **origine** (conception d'origine, étape 0, ou 29-4b).
3. Décompte final par sévérité, recompté depuis la liste.

---

## Lentille B — Blind Hunter (le diff seul, sans la fiche)

Lis **seulement** le diff (`git diff fe404b11..HEAD -- crates CHANGELOG.md docs/api-external.md
docs/manual/fr/user-manual.tex`) et les fichiers qu'il touche ; **ne lis pas** la fiche ni le registre avant d'avoir
écrit tes findings (tu peux les ouvrir ensuite pour reclasser, en le disant). Axes :

- correction des cinq appels : bon compte (W9 : `bt.bank_account_id` ; W25 : `settlement_bank_account_id` ; W28,
  W29 : `inv.settlement_bank_account_id`, lu **avant** l'`UPDATE` qui le remet à `NULL` ; W24 :
  `import.bank_account_id`), condition « si le chemin porte un compte bancaire », position après la **dernière**
  écriture et avant l'audit ; l'erreur rendue (`BankAccountClosed`) et sa traduction par chaque route (code et
  `details`) ; un `NotFound` de la revérification est-il atteignable, et que rendrait-il ?
- `primary_flag_locked_in_tx` et `archive_once` : ordre des refus inchangé (412 transactions → 412 principal → 409
  version → 404) ; `None` → `404` : juste ? la valeur du pré-vol subsiste-t-elle quelque part ?
- tests : un test qui passerait **à vide** (sonde jamais exercée ; `annuler_pendant_une_cloture` qui ignore le
  résultat de `attendre_une_requete_en_cours` — est-ce un trou ?) ; tâches non jointes, transactions laissées
  ouvertes ; l'instantané `etat` couvre-t-il tout ce que chaque annulation écrit ?
- le registre (`bank_account_closed_inventory.rs`) : `Position::EnDernier` et `fautes_en_dernier` — un geste
  verrouillant que la liste ne voit pas dans l'un des cinq corps ? `UPDATE ` qui manquerait un littéral ? la copie
  masquée efface-t-elle bien chaînes et commentaires ? le volet inverse rougit-il sur chaque faute, et sur elle
  seule ? T29b-7 (textuel) : que prouve-t-il, que ne prouve-t-il pas ?
- doc-comments : chaque phrase neuve est-elle vraie du code ?

## Lentille E — Edge Case Hunter (verrous, cycles, naissance de la vue)

⛔ **Table obligatoire** dans le rapport : **la table des verrous et des cycles**. Pour chaque chemin touché — W9
(annulation de rapprochement, `Entry` et règlement imbriqué W25), W25, W28, W29, W24, et l'archivage
(`archive_once`, R-2) — la suite **ordonnée** des verrous pris (ligne, mode S/X, explicite ou par clé étrangère,
intervalle ou non, verrou nommé `GET_LOCK` pour W9), et pour chaque verrou **neuf** ou **avancé** : les partenaires
qui peuvent fermer un cycle avec lui (à deux, à trois par la file d'attente InnoDB, à quatre), si chaque participant
est rejoué, et qui InnoDB désignerait pour victime (argument de poids : écritures et verrous). Compare ta table à
celle de la fiche (Dev Notes § *Position par chemin* et § *Cycles*) et dis chaque écart. Axes :

- **le critère décidable** de la fiche, trois volets : (1) le verrou neuf de chaque annulation est la **dernière**
  acquisition de sa transaction (seuls le suivent l'audit et des lectures non verrouillantes — W9 : la relecture
  `find(…, false)` ; W28, W29 : `FIND_SCOPED_SQL`, `fetch_lines`) — W9 qui imbrique W25 : deux revérifications,
  sur le même compte ou deux comptes ; (2) tout cycle fermé a ses participants rejoués (W8 `settle_invoice`, W27
  `pay_in_tx` par `pay` et `confirm_batch`, la clôture, la réouverture, le `PATCH`, les routes de gestion, la
  restauration) — la route de W24 (`bank_imports::delete`) et celles des quatre autres annulations sont-elles toutes
  rejouées ? (3) **aucune naissance de vue déplacée** ;
- W9 et le verrou nommé du compte : la revérification attend en gardant `GET_LOCK` ; un cycle mixte verrou nommé /
  verrou de ligne (non détecté par InnoDB, `1205`) est-il exclu ?
- R-2 : l'exclusif de la ligne du compte **avancé** devant `count_other_active_for_company` — un cycle neuf ? la
  prémisse « entre les deux positions, aucune lecture verrouillante » est-elle vraie ?
- T29b-3 : la victime observée est-elle prouvée (événement `invoices::cancel_settlement`, `.expect` de la
  transaction de test dans `victime()`) ? T29b-1 et T29b-2 : que se passe-t-il si l'annulation **attend** ailleurs
  que sur sa revérification (au sol, elle n'attend pas) ?
- `innodb_snapshot_isolation` (angle mort renvoyé à #669) : le doc-comment de `bank_imports.rs` (étape 5) le dit-il
  juste ? les quatre autres annulations ont-elles le même rayon, et est-il écrit ?

## Lentille A — Acceptance Auditor (la fiche contre le code)

Lis la fiche (partie b, T6-bis, T7 à T11), le registre des choix et le Dev Agent Record, puis le diff. Axes :

- **T6-bis** : R-1 (T29a-15 dit synthétique, au test et au point (vi)), R-2 (relu dans la transaction, test rouge
  d'abord : journal `294b-rouge-R2.log`), R-4 (mesuré / supposé distingués), R-5 (mutation ou « non couvert »
  déclaré), `docs/api-external.md` § *Interblocages* (les routes nommées existent-elles, et sont-elles toutes
  rejouées au code ?) ;
- chaque AC de la partie b (AC6 à AC8, AC9 part b, AC10) : satisfait, partiellement, ou non — preuve par `grep -nF` ;
  chaque écart à la fiche est-il **écrit** (registre C-29-4-32 à C-29-4-37, Dev Agent Record) ?
- les tests prescrits (T29b-1 à T29b-5) : présents, au bon fichier ou déplacés avec leur choix (C-29-4-32), et
  éprouvant ce que la fiche dit ; T-a31, T-a48, `a_concurrent_close_waits_for_the_unreconciliation`,
  `annuler_un_rejet_pendant_une_cloture_refuse_par_w30` restés **sans retouche** ;
- la campagne de mutations (M29-8 à M29-18, M29-24, M29-35) : le Dev Agent Record donne-t-il pour chacune « tuée
  par … » ou « équivalente, démontrée », adossé au journal `294b-mutations.log` ?
- la table *Position par chemin* et la table des cycles recopiées au Dev Agent Record avec les lignes retenues,
  contre le code ;
- **les textes de L1** (AC9 part b) : chaque site prescrit réécrit (doc-comments de `bank_accounts.rs` — module,
  § fenêtre, `ensure_open_unarchived_in_tx` —, `bank_imports.rs` (`innodb_snapshot_isolation`), registre l. 95,
  121, 241, 298, raison de W30, `routes/reconciliation.rs` `closed_account_failed`, `bank_accounts_closed_e2e.rs`
  en-tête et W21) ; la propagation : `grep -rnE "651|\bL1\b" crates docs CHANGELOG.md frontend/src website
  README.md` et `grep -rnE "par une course|hors de cette fenêtre|peut encore aboutir"` — chaque site restant trié ;
- le **manuel** (`.tex` **et PDF aplati**) et `docs/api-external.md` : le texte prescrit, et vrai du code ; la
  section `[0.13.3]` du CHANGELOG non touchée ; la ligne #651 du § `[0.13.4]` complétée par union ;
- les gates déclarés : chaque chiffre du Dev Agent Record adossé à un journal de `/home/gcorbaz/devel/kesh-gate-logs/`
  (`294b-gate-*.log`, `294b-fe-*.log`, `294b-e2e*.log`) ; rien de déclaré qui n'ait tourné ; les décomptes (tests
  neufs, fichiers, choix) recomptés ;
- le **décompte des modules** de la dérogation de splitting (la fiche annonçait quatre modules de code pour la
  29-4b ; T6-bis en ajoute) : écrit, et juste ?
- aucun mot-clé de fermeture dans les messages de commit
  (`git log fe404b11..HEAD --format=%B | grep -niE '(clos(e|es|ed)|fix(es|ed)?|resolv(e|es|ed)):? *#[0-9]'` — le
  type conventionnel `fix(29-4b):` n'en est pas un) — seul `refs #651` est permis dans les commits ; `closes #651`
  est réservé au corps de la PR.
