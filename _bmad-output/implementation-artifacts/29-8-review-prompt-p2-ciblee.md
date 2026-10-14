# Prompt — revue de code P2, passe ciblée, Story 29-8 (une transaction réglée depuis la fiche de la facture se rattache à ce règlement)

*Versionné le 2026-10-14. **Une seule lentille**, contexte frais, lecture seule stricte. Modèle : **celui qu'indique
l'orchestrateur** (passe ciblée : Haiku admis, décision D6 de la rétrospective 25). Elle ne relit pas la story : elle
relit **la seule remédiation de la revue P1**, là où se loge, sur cet epic comme sur les précédents, le défaut suivant
(`CLAUDE.md`, § *La passe ciblée*).*

Dépôt `/home/gcorbaz/devel/kesh-29-8`, branche `story/29-8`. Le diff à relire :
`git -C /home/gcorbaz/devel/kesh-29-8 diff 4b358373 HEAD` (`4b358373` = dernier commit avant la remédiation P1 ; le
diff ne contient qu'elle). Pour un fichier, `git diff 4b358373 HEAD -- <fichier>` ; pour l'état final, `git show
HEAD:<fichier>` — ⚠️ ne lis pas les numéros de ligne d'un hunk comme ceux du fichier final. Langue du rapport : français.

Contexte : rapports de la revue P1, `/home/gcorbaz/devel/kesh-gate-logs/298-review-p1-{B,E,A}.md` ; fiche
`_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md` (Dev Agent Record, § *Remédiation de la
revue de code P1* ; Change Log, entrée « Revue de code P1 ») ; registre
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-8.md`, **C-29-8-37 à C-29-8-41** et les annotations
datées de C-29-8-25 et C-29-8-35. Règles : `CLAUDE.md` (§ Review Iteration Rule et sous-sections ; § *Un appariement
automatique propose, il ne crée jamais* ; § Migration breaking policy — **aucune** migration attendue) ;
`docs/agents-orchestration.md` § 6.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée **aucun** fichier — ni temporaire, ni dans `/tmp`, ni dans le
  dépôt —, hormis ton rapport.
- **N'exécute rien qui écrive, compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni
  `make`, ni `latexmk`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou du répertoire de gate (dont `298-mutations-p1.py` et `298-mutations-p1-front.py`) ; ni `python3`,
  ni `perl`, ni `sed -i`, ni `docker`, ni `mariadb`.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni de fichier, ni de branche, ni `--detach`), aucun `git`
  qui écrit (`commit`, `stash`, `reset`, `restore`, `rebase`, `tag`, `push`, `worktree`). Aucune commande `gh` qui
  écrit ; `gh issue view` est permis.
- Aucune redirection de sortie vers un fichier autre que ton rapport (`pdftotext` vers la sortie standard seulement).
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext … -`.

## La lentille — la remédiation dit-elle vrai, et n'a-t-elle rien cassé ?

Axes, **tous obligatoires** :

1. **L'ordre des refus écrit contre le code.** La table réécrite de `docs/api-external.md` (§ *Rattacher une
   transaction à un règlement déjà enregistré*, « Refus, dans cet ordre ») et le doc-comment de
   `post_link_transaction` (`crates/kesh-api/src/routes/reconciliation.rs`) : relis, dans le code, `lib.rs` (groupe de
   routes et middleware : rôle, clé `read`), `post_link_transaction` (extraction du corps, `400`),
   `bank_account_of_transaction`, `link_preflight`, `link_once` et `with_account_lock`, puis `link_in_tx`
   (`crates/kesh-db/src/repositories/reconciliation_link.rs`, étapes 4.1 à 5). Chaque ligne de la table est-elle à son
   rang ? Une ligne manque-t-elle, ou en reste-t-il une que le code n'atteint pas (au `GET` ou au `POST`) ? Le `GET`
   s'arrête-t-il bien où la table le dit ? Un rang faux dans un document externe qui s'annonce « dans cet ordre » est au
   moins **MEDIUM**.
2. **La garde de session de la modale** (`frontend/src/lib/features/reconciliation/LinkSettlementModal.svelte`) :
   `session` et `loadGen` (non `$state`), l'`$effect` (ce qu'il lit, ce qu'il écrit, s'il peut boucler ou ne pas se
   relancer ; `open ? proposal : null`), `handleConfirm` (succès, refus et `finally` d'une session périmée ; `if
   (!open) onSuccess()` — lit-il la valeur courante de la prop ?), `RELIRE_SUR` (que se passe-t-il quand `NOT_FOUND`
   vise la **transaction** disparue ? quand `NOT_PENDING` est relu ?), `valueDate ?? bookingDate` (et une chaîne vide ?).
   Un résultat d'une session périmée qui atteint encore la modale courante, ou un succès réel que le parent n'apprend
   jamais alors qu'aucune modale n'est ouverte, est au moins **MEDIUM**. Lis le parent
   (`ReconciliationProposals.svelte`, `onLinkSuccess`, `onOpenChange`) pour juger des effets.
3. **Les tests neufs prouvent-ils ce qu'ils disent ?**
   - `t17bis_le_second_rattachement_attend_le_verrou_du_reglement` (`crates/kesh-api/tests/reconciliation_link_e2e.rs`)
     : les motifs `["invoice_settlements", "FOR UPDATE"]` de `attendre_une_requete_en_cours`
     (`crates/kesh-db/src/test_fixtures.rs`) ne désignent-ils que la requête de l'étape 4.2 (a) ? La connexion `tenue`
     peut-elle être vue par la sonde ? L'`UPDATE` de `tenue` peut-il attendre un verrou du `POST` (interblocage) ?
     Rougit-il réellement sous M10 et M14 — relis `/home/gcorbaz/devel/kesh-gate-logs/298-mut-p1-M10.log` et
     `298-mut-p1-M14.log`, et `298-mutations-p1-result.txt` ?
   - T13 (a) (témoin rapproché, M20) et T20 (archivé puis clos, M21) : rougiraient-ils si l'ordre s'inversait
     (`298-mut-p1-M20.log`, `298-mut-p1-M21.log`) ?
   - Vitest (`LinkSettlementModal.test.ts`, huit tests neufs) : `rerender` de `@testing-library/svelte` 5 change-t-il
     les props comme le test le suppose ? `vider()` suffit-il à laisser passer les continuations, ou un test peut-il
     être vert **avant** que le code observé n'ait tourné (test muet) ? Relis `298-p1-vitest-red.log` (les tests neufs
     contre l'ancienne modale) et `298-mut-p1-MF*.log`.
   - Spec E2E `frontend/tests/e2e/reconciliation-link.spec.ts` : la regex du compteur, l'`href` du lien,
     `matchedEntryId === journalEntryId`.
4. **Les textes contre le code** : le message `reconciliation-errors-settlement-not-linkable` dans les **quatre**
   `crates/kesh-i18n/locales/*/messages.ftl` et le repli d'`errors.rs` (égalité mot pour mot avec le fr-CH, garde G9 de
   `textes_coherents.rs`) ; chaque cause qu'il nomme est-elle une cause réelle de ce code, et en omet-il une ? Les
   traductions disent-elles la même chose que le fr-CH ? Même question pour la table des erreurs
   (`docs/api-external.md`, ligne `RECONCILIATION_SETTLEMENT_NOT_LINKABLE`), la note ⁹ (gestes qui peuvent rendre `409
   RECONCILIATION_ACCOUNT_LOCKED`) et le paragraphe de l'annulation de rejet (« l'API permet aussi sans annuler le
   rejet » : vrai au code ?).
5. **Manuel contre code (obligatoire), et le PDF** : `docs/manual/fr/user-manual.tex`, puce *Rejeter* (« … éclatée ou
   rattachée à un règlement déjà enregistré ») et paragraphe *Le geste* de `sec:rattacher-reglement` (date de paiement
   rappelée ; « Kesh le dit et relit la liste ; rien n'est rattaché » — sur quels refus exactement ?) ; **et**
   `docs/manual/fr/user-manual.pdf`, aplati et normalisé :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g;s/’/'/g"`
   — les phrases neuves y sont-elles, sans « ?? » ? Une promesse du manuel que le code ne tient pas est au moins
   **MEDIUM**.
6. **Propagation** : greper les **valeurs** corrigées sur `crates`, `frontend/src`, `frontend/tests`, `docs/`,
   `CHANGELOG.md`, `website/` — l'ancien message (`est sur un autre compte bancaire`, et ses trois traductions), les
   énumérations fermées des gestes (`manuellement ou éclatée`, `annulation d'un autre rapprochement`), l'ancien ordre
   de la table (`Règlement introuvable ou d'une autre société`), `bookingDate` dans la modale. Reste-t-il une copie du
   symptôme ?
7. **Le compte rendu** : le Change Log « Revue de code P1 », le § *Remédiation* du Dev Agent Record, les choix C-29-8-37
   à 41 et les annotations datées : chaque chiffre (tests neufs, mutations, gates, E2E et ses échecs jugés contre
   `docs/testing.md` § *Les échecs attendus*) se recompte-t-il depuis la source et les journaux
   `/home/gcorbaz/devel/kesh-gate-logs/298-p1-*` ? Une entrée de passe passée a-t-elle été réécrite au lieu d'être
   annotée ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/298-review-p2-T.md` :

- findings numérotés `T1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne (du fichier
  **final**), preuve (**sortie `grep -nF` ou `sed -n` copiée** pour toute affirmation de présence ou d'absence),
  correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été**, axe par axe — un « 0 finding » sans cette
  liste ne compte pas ;
- rien d'autre n'est écrit nulle part.
