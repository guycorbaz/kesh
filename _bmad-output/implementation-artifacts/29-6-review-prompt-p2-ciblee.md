# Prompt — revue de code P2, passe ciblée, Story 29-6 (une règle ne remplace plus en silence une facture trop-perçue)

*Versionné le 2026-10-14. **Une seule lentille**, contexte frais, lecture seule stricte. Modèle : **Sonnet** (décision
de l'orchestrateur). Elle ne relit pas la story : elle relit **la seule remédiation de la revue P1**, là où se loge, sur
cet epic comme sur les précédents, le défaut suivant (`CLAUDE.md`, § *La passe ciblée*).*

Dépôt `/home/gcorbaz/devel/kesh-29-6`, branche `story/29-6`. Le diff à relire :
`git -C /home/gcorbaz/devel/kesh-29-6 diff 4b34d45e HEAD` (`4b34d45e` = dernier commit avant la remédiation P1 ; le
diff ne contient qu'elle). Langue du rapport : français.

Contexte : rapports de la revue P1, `/home/gcorbaz/devel/kesh-gate-logs/29-6-review-p1-{B,E,A}.md` (lis surtout
**E1**) ; fiche `_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md` (Change Log, entrée « Revue de
code P1 ») ; registre de l'epic `_bmad-output/implementation-artifacts/epic-29-choix-autonomes.md`, **C-29-6-24 à
C-29-6-27** (versés depuis le registre propre de la story, retiré). Règles : `CLAUDE.md` (§ *Un appariement automatique
propose, il ne crée jamais* ; § Review Iteration Rule et sous-sections ; § Pattern batch ; § Migration breaking policy
— aucune migration attendue) ; `docs/agents-orchestration.md` § 6.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée **aucun** fichier — ni temporaire, ni dans `/tmp`, ni dans le
  dépôt —, hormis ton rapport.
- **N'exécute rien qui écrive, compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni
  `make`, ni `latexmk`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou du répertoire de gate ; ni `python3`, ni `perl`, ni `sed -i`, ni `docker`, ni `mariadb`.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni de fichier, ni de branche), aucun `git` qui écrit
  (`commit`, `stash`, `reset`, `restore`, `rebase`, `tag`, `push`, `worktree`). Aucune commande `gh` qui écrit ;
  `gh issue view` est permis.
- Aucune redirection de sortie vers un fichier autre que ton rapport (`pdftotext` vers la sortie standard seulement).
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext … -`.

## La lentille — une règle ne remplace plus jamais en silence une facture trop-perçue

La propriété à éprouver : **sur toute entrée atteignable, une facture que le paiement dépasse de 1 à 5 centimes et
qui, avant la story, aurait été proposée devant la règle d'affectation (ou l'aurait empêchée), empêche toujours la
règle d'être `candidates[0]`** ; et, réciproquement, **la remédiation ne masque pas une règle que la présence de la
facture ne masquait pas avant la story**. Avant la story : `find_unpaid_invoices_for_window` chargeait les restes dans
`tx ± 0.05`, `propose_matches` écartait les factures de score total nul, une facture de score ≥ 0,5 empêchait le calcul
de la règle, et `order_candidates` plaçait toute facture à indice devant la règle.

Axes, tous obligatoires :

1. **La requête neuve** — `find_invoices_slightly_overpaid_for_window` et le fragment commun
   `unpaid_invoices_in_window_sql` (`crates/kesh-db/src/repositories/reconciliation.rs`) : la clause `HAVING`
   (`amount_due >= ? - ? AND amount_due < ?`) et l'**ordre de ses binds** ; les deux intervalles (candidates et
   trop-perçus) sont-ils disjoints et leur réunion égale-t-elle exactement l'ancien `tx − 0.05 … tx + 0.05` ? La
   factorisation laisse-t-elle `find_unpaid_invoices_for_window` **strictement** inchangée (texte SQL rendu, binds,
   `ORDER BY`, `LIMIT 50`) ? Une différence de ce texte est au moins **HIGH**.
2. **La route** — `get_proposals` (`crates/kesh-api/src/routes/reconciliation.rs`) : les règles actives lues avant le
   Pass 2 ; la lecture des trop-perçus seulement si une règle existe ; leurs contacts ajoutés au chargement groupé ;
   `rule_masked_by_overpaid_invoice` calculé par `propose_matches` sur ces seules factures ; la condition de la règle.
   Cherche une voie par laquelle la règle redevient `candidates[0]` alors qu'une facture trop-perçue à indice existe
   (fenêtre de dates, `LIMIT 50` des trop-perçus, contact absent, devise, débit, montant nul, règle dont le compte est
   exclu), ou par laquelle elle disparaît sans qu'aucune facture à indice ne la masque. Le gate sur
   `active_rules.is_empty()` change-t-il un comportement ? Le refactor de `with_contacts` change-t-il le score des
   candidates ? Une voie atteignable qui rouvre le faux rattachement muet est au moins **HIGH**.
3. **Les tests et les mutations** — `a_rule_never_replaces_an_invoice_the_payment_slightly_exceeds`,
   `the_rule_guard_needs_an_invoice_bearing_a_signal` (`crates/kesh-api/tests/reconciliation_e2e.rs`),
   `find_unpaid_invoices_bounds_hold_for_a_one_centime_payment`,
   `find_invoices_slightly_overpaid_returns_what_the_lower_bound_excludes`
   (`crates/kesh-db/tests/reconciliation_repository.rs`) : attendus en dur ? chaque assertion éprouve-t-elle ce que
   son message annonce (scores attendus des factures : montant, référence, contact) ? Un test vert par construction,
   ou qui passerait sans le correctif, est au moins **MEDIUM**. Journaux : `29-6-p1-red.log` (rouge sans correctif),
   `29-6-p1-mut-{baseline,M14,M15,M16,M17}.log` — chaque mutation est-elle tuée par le test que le compte rendu nomme ?
   Reste-t-il une mutation plausible de la remédiation qu'aucun test ne tuerait ?
4. **Manuel contre code (obligatoire)** — les phrases neuves de `docs/manual/fr/user-manual.tex` (§ *Algorithme de
   matching*, paragraphe sur la règle proposée « quand aucune facture n'est candidate », liste de l'écran, § *Règles
   d'affectation automatique*, FAQ `sec:faq-aucun-match` et son demi-centime) **et le PDF**
   `docs/manual/fr/user-manual.pdf`, aplati et normalisé :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g;s/’/'/g"` ;
   `docs/api-external.md` (ordre de `candidates`, la règle absente) ; `CHANGELOG.md` (entrée #665). Disent-ils
   **exactement** la condition du code — facture **désignée** (référence liée à son numéro, contrepartie liée au nom de
   son client), dépassée de 5 centimes **au plus**, et non toute facture trop-perçue ? Une promesse que le code ne
   tient pas, ou une condition plus large ou plus étroite que le code, est au moins **MEDIUM**.
5. **Propagation** — greper les valeurs : `aucune facture`, `n'est candidate`, `sinon la règle`, `otherwise the
   matching`, `rule fallback`, `slightly_overpaid`, `rule_masked`, `± 0.05` sur `crates`, `frontend/src`,
   `frontend/tests`, `docs/`, `CHANGELOG.md`, `website/`. C-29-6-26 laisse la phrase de priorité de l'écran
   simplifiée : la justification tient-elle ?
6. **Le compte rendu** — l'entrée « Revue de code P1 » du Change Log, le Dev Agent Record et C-29-6-24 à 27 :
   chaque chiffre (tests, gates, mutations) se recompte-t-il depuis la source et les journaux
   `/home/gcorbaz/devel/kesh-gate-logs/29-6-p1-*.log` ? Les LOW B1, B2, A1 à A4, E2 sont-ils appliqués comme annoncé,
   et B3 = E3 écrit comme limite (C-29-6-25) de façon défendable ? Les entrées datées des passes précédentes sont-elles
   restées intactes (annotations seulement) ?

Hors axe, ne pas relire : la règle de la story elle-même (borne basse, `classify_payment`), l'écran (case, *Accepter*
grisé) — relus en P1 sans finding au-dessus de LOW.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/29-6-review-p2-ciblee.md`. Il porte :

- le décompte `CRITICAL / HIGH / MEDIUM / LOW` ;
- chaque finding avec sa sévérité, le fichier et la ligne, la **preuve** (sortie `grep -nF` ou extrait `sed -n` copié)
  et un correctif proposé ;
- la liste des **axes exercés** et celle des **axes non exercés** (ou exercés en partie), avec la raison — un
  « 0 finding » sans cette liste ne compte pas comme passe ;
- toute affirmation de présence ou d'absence s'appuie sur une sortie copiée, jamais sur une estimation.
