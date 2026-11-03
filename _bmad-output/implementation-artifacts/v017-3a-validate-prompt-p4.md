# Prompt — validation P4 (ciblée) de la fiche v017-3a (#671 : rattacher une transaction à une écriture existante), une lentille, Opus

Tu valides la **remédiation de la passe P3** de la fiche de la Story v017-3a de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11). Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur le
**seul diff** du commit `docs(v017-3a) : remédiation de la validation P3 (refs #671)`. Modèle : **Opus**. **Contexte
frais** : ne t'appuie sur aucun rapport antérieur pour juger, ni sur ce que la fiche dit d'elle-même. Langue du rapport :
**français**.

Worktree `/home/gcorbaz/devel/kesh-0173`, branche `story/v017-3-rapprochement-attente`, partie de `c5101b07`.
Historique : P1 (Sonnet) 0/1/12/12 ; P2 (Opus) 0/0/12/14, suivie d'un **découpage** (C-v017-3-25 : l'avertissement des
propositions est parti à la v017-3c, #828) ; P3 (Sonnet) 0/0/6/12.

## À lire

1. Le diff ciblé : `git log --oneline c5101b07..HEAD`, puis `git show <commit de la remédiation P3>` (fiche
   `v017-3a-rattacher-ecriture-existante.md`, fiche `v017-3c-avertissements-propositions.md`, registre
   `v017-3-choix-autonomes.md`).
2. La fiche 3a entière **après** le diff, pour juger chaque ajout dans son contexte.
3. Les rapports P3, **après** ton examen : `/home/gcorbaz/devel/kesh-gate-logs/v017-3a-validate-p3-R.md` et `-F.md`.

## La lentille — chaque prémisse introduite, vérifiée au code

Pour **chaque** `fichier:ligne`, nom de fichier, de fonction, de clé, de test ou de constante que le diff introduit,
ouvre la source et cite la ligne lue. Au minimum :

- **AC-3 (b) réécrit** : `crates/kesh-api/src/routes/bank_imports.rs:1507-1518` (`delete_once` sans `with_account_lock`) ;
  `crates/kesh-db/src/repositories/journal_entries.rs:2278` (la lecture de `bank_transactions` par `reversal_blockers`
  est-elle **sans** verrou ?), `:2741-2745`, `:1457-1459`, `:1831-1835` ; `reconciliation_cancel.rs` (ordre `bt` puis
  `je`). La nouvelle affirmation — « aucun geste qui tient une ligne `journal_entries` ne verrouille ensuite
  `bank_transactions` » — est-elle **vraie** ? Cherche un contre-exemple, **par fonction** (pas par grep mono-ligne) :
  confirmation d'un lot de paiement, acceptation d'un lot de rapprochement (lettrage puis `UPDATE` de la transaction
  suivante), annulation d'un règlement client ou fournisseur, clôture d'un compte bancaire, suppression d'une facture.
  Le critère d'arrêt de l'étape 0 est-il décidable tel qu'écrit ?
- **Coût de `cancelBlockedBy` (AC-5)** : l'estimation (« huit à douze requêtes » par candidate) tient-elle à la lecture
  de `settlement_cancellation.rs:87-182` et du classement ? Le seuil (1 s, 50 candidates) est-il mesurable sur la base
  de test, et le montage décrit suffisant ?
- **Le dialogue (AC-13)** : le gabarit (clé d'introduction, phrase complète, paragraphe distinct) compose-t-il un texte
  juste avec **chacun** des huit textes de `reconciliationCancelMessage` (`reconciliation-cancel.ts:38-91`) ? La phrase
  sans motif est-elle vraie au regard de L9 ? Les quatre cas de listes vides sont-ils disjoints et exhaustifs (crédit,
  débit ; règlements, écritures), et leurs textes vrais ? Les clés neuves portent-elles le préfixe `reconciliation-link-`
  que contrôle `LinkSettlementModal.test.ts:357-370` ?
- **`LinkSettlementModal.test.ts`** : `:12-15`, `:123`, `:216`, `:357-370` disent-ils ce que la fiche leur fait dire ?
- **Manuel (AC-18)** : les points 13 à 16 et la table de tri du filet `rattach` — chaque ligne citée (`:264` à `:3486`)
  dit-elle ce que la table lui attribue ? Une ligne est-elle mal classée « inchangée » ou « sans rapport » ? Les sorties
  copiées (44 lignes `.tex`, 45 fragments PDF) se reproduisent-elles (`grep -n -i "rattach" docs/manual/fr/user-manual.tex`
  ; `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oi ".\{40\}rattach.\{40\}"`, sortie
  standard seulement) ? La liste est-elle **close** ?
- **Collisions** : `README.md:41` (texte exact, balisage), `frontend/src/lib/shared/i18n-keys.test.ts:767`,
  `docs/api-external.md:243`, `:878` ; `/home/gcorbaz/devel/kesh-0162/_bmad-output/implementation-artifacts/v016-2a-releve-manuel-backend.md`
  (`:226-229`, `:397`) et `v016-2b-releve-manuel-ecran.md` (`:109`, `:203`) — en lecture seule.
- **CHANGELOG (F8)** : `CHANGELOG.md:343-346` est-il bien le précédent de la 29-8 en *Ajouté* ?
- **Mutations M20, M27 à M30** : chacune rougit-elle le test annoncé, pour la raison annoncée ?
- **Issues et registre** : #827, #828 (`gh issue view`) ; la fiche 3c ; les deux amendements du registre sont-ils placés
  **sous** l'entrée qu'ils amendent (C-v017-3-10, C-v017-3-21), datés, sans réécrire l'entrée d'origine ?
- **Propagation** : le symptôme corrigé (« même verrou nommé », « chaque motif se lève », « environ soixante », « trois
  cas », « à ouvrir », `:653`, `:198-201`) a-t-il été grepé dans la fiche, le registre et la fiche 3c ? Cite toute
  occurrence restante non marquée comme historique.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v017-3a-validate-p4.md`. Il
contient :

1. **Les findings**, numérotés `C1`, `C2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable ». Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime
  pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh-0162` ni dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
