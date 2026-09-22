# Prompt — passe 3 de `bmad-create-story validate`, Story 25-2-b

*Versionné le 2026-09-20. Une lentille en contexte frais (Sonnet) — la passe 2 était sur Opus, la
passe 1 sur Sonnet et Haiku 4.5. ⚠️ L'auteur des patches est un Opus : ne fais AUCUNE confiance au
fait qu'un passage « a déjà été revu deux fois ».*

Tu es un **valideur adversarial** en contexte frais. Ton objet est
`_bmad-output/implementation-artifacts/25-2-b-devalidation-facture.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-2-b-devalidation-facture`. Ta mission n'est pas
d'approuver : c'est de **trouver ce qui ferait échouer, dévier ou mentir** l'implémentation.

⛔ **Rien ne se croit sur parole.** Chaque `fichier:ligne`, chaque signature, chaque comportement se
vérifie dans le code. ⚠️ La branche est tirée d'un `main` qui a reçu #439, #441 et #442 ; la
**25-2-b-zero** (#443, PR #444, **non mergée**) pose la garde du verrou de période dont l'AC 3
dépend : lis son code sur la branche `story/25-2-b-zero-verrou-periode-suppression`.

## Les arbitrages — à ne PAS contester, seulement leur mise en œuvre

Guy, 2026-09-16 et 2026-09-19 : la dévalidation existe (la contre-passation est inapplicable) ;
deux cycles, effacer **et** corriger-revalider ; `emailed_at` en **refus sec** ; le numéro de
facture **conservé** ; rôle **Administrateur et Comptable** ; **clés API admises** (« même approche
que Bexio ») ; un brouillon numéroté **ne change pas d'exercice** ; l'**asymétrie** des deux sorties
est gardée (seul l'Administrateur efface) ; la garde de période est sortie en 25-2-b-zero.

## Ce qui a changé depuis la passe 2, et où regarder d'abord

Les patches (`c38c92f1`, `9f5d73df`, et ceux de la passe 2) ont réécrit l'**AC 2** (exercice du
numéro conservé, code `INVOICE_NUMBER_FISCAL_YEAR_MISMATCH`), l'**AC 3** (huit empêchements, enum à
cinq variantes sur le modèle de `ReversalBlocker`, précédence **1-4, 7, puis 5, 8, 6**, fenêtre de
l'envoi d'e-mail), l'**AC 4** (trou dans la séquence des factures, asymétrie des rôles), l'**AC 9**
(rôle, clés API, friction), l'**AC 10** et l'**AC 11**, et les tâches T1-T2.

⛔ *La sévérité se déplace vers ce qu'on vient d'écrire.* Commence par ces passages. En particulier :

1. **La garde d'exercice de l'AC 2.** Est-elle réalisable telle qu'écrite — où se pose-t-elle dans
   `PUT /api/v1/invoices/{id}` et dans la revalidation, que fait le code aujourd'hui de la date d'un
   brouillon, et un brouillon **sans** numéro doit-il en être exempté ? Le code neuf est-il cohérent
   avec les conventions (nom, statut HTTP, i18n) ?
2. **La précédence 1-4, 7, puis 5, 8, 6.** Est-elle réalisable sans dupliquer des lectures entre
   `unvalidate` et `delete_in_tx` ? Un test à deux empêchements est-il constructible de chaque côté ?
3. **La fenêtre de l'envoi d'e-mail** (marquer avant d'expédier, retirer la marque en cas d'échec) :
   que casse ce renversement — tests, écran, journal d'audit, rappels (`record_reminder_email`) ?
   Une facture restée marquée par un échec devient-elle **indévalidable** à jamais ?
4. **Le numéro conservé.** `validate_invoice` ne tire du compteur que si `invoice_number` est absent :
   quels autres sites lisent ou écrivent `invoice_number`, et qu'attendent-ils d'un **brouillon
   numéroté** (unicité, QR-facture, PDF, export, sauvegarde, rappels, listes, filtres) ?
5. **L'audit et les registres.** `invoice.unvalidated` au registre, `SITES_INDIRECTS` (l'entrée de
   `invoices.rs` **existe déjà**), `audit_route_registry` et ses totaux en dur, la route dans
   `comptable_routes` et les clés API.
6. **Les manuels et `docs/api-external.md`** : la liste des sites de l'AC 10 est-elle close ?
   Contrôle les **PDF aplatis** des deux manuels FR. Cherche un site que la spec ne nomme pas.
7. **Tests, décomptes, périmètre, découpage.** Les cinq tests existants nommés à l'AC 5 sont-ils les
   bons et les seuls ? La story franchit-elle le seuil de la *Règle de splitting* ?

## Ce que tu rends

- **Les findings** : sévérité, l'endroit exact, **la commande ou l'extrait qui l'établit**, ce qu'il
  faut changer. Pour un scénario, montre que son état de départ est atteignable.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un reproche au code de
  ne pas encore faire ce que la story prescrit n'est **pas** un défaut de la spec.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une
base persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans
`docs/manual/`, tout `git commit`/`push`/`add`/`stash`/`reset`/`rebase`, `sqlx migrate` sur `kesh`
ou `kesh_e2e`, `cargo test`/`cargo nextest`. Lecture, `grep`, `git log`/`show`/`diff`,
`git checkout` d'une AUTRE branche en lecture seule (reviens sur `story/25-2-b-devalidation-facture`
ensuite), `gh issue view`, `gh pr view`, `pdftotext` et `cargo check` sont autorisés.
