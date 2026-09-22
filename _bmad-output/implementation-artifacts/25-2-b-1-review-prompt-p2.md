# Prompt — passe 2 de `bmad-code-review`, Story 25-2-b-1

*Versionné le 2026-09-22. Deux lentilles, en contexte frais, sur des modèles différents de la
passe 1 (qui était Sonnet ×3) : **R** sur Opus, **D** sur Haiku 4.5.*

**Pourquoi deux et non trois.** La passe 1 a rendu `3 HIGH, 4 MEDIUM, 3 LOW`, et **les trois HIGH
portaient sur ce que la story croyait déjà tenir** — deux sur son propre compte rendu. La
remédiation qui a suivi touche **du code de production** (la signature de `invoices::unvalidate`, le
handler HTTP), donc la boucle ne peut pas se clore ; et le motif documenté au `CLAUDE.md` — *la
sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* — désigne l'endroit à relire
en priorité. D'où une lentille **braquée sur la remédiation** et une lentille **sur le périmètre
complet**, plutôt que trois lentilles rejouant la passe 1.

## Préambule commun

Tu es un **relecteur adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`, branche
`story/25-2-b-1-devalidation-depot-api`. La spec est
`_bmad-output/implementation-artifacts/25-2-b-1-devalidation-depot-api.md` ; la fiche mère
`25-2-b-devalidation-facture.md` est la source des faits établis et ne se rediscute pas.

⛔ **Rien ne se croit sur parole** — ni la spec, ni les commentaires, ni le Change Log, ni les
messages de commit, **ni le rapport de la passe 1 qui y est résumé**. Chaque affirmation se vérifie
dans le code actuel.

⛔ **Grep ground-truth obligatoire.** Tout finding `CRITICAL` ou `HIGH` affirmant **l'absence d'un
code attendu** ou **la présence d'un anti-pattern** se vérifie avant d'être rendu, par
`grep -nF "<chaîne exacte>" <fichier>` — le `-F` est **obligatoire**, le code Rust est plein de
métacaractères (`Vec<i64>`, `unwrap_or(0.0)`, `*`, `(`). Pour un bloc, `grep -nFA 5`. Pour un flux
cross-fonction sans motif discriminant, lecture directe, et **cite l'extrait lu**.

⚠️ **Ne conteste pas les arbitrages** : `comptable_routes` (Administrateur **et** Comptable) ; les
clés API passent en écriture (« même approche que Bexio ») ; `emailed_at` est un refus sec non
levable ; le retrait de la branche `validated` d'`invoices::delete`, l'écran, les manuels et le
`CHANGELOG` appartiennent à la **25-2-b-2** et sont **hors périmètre**. Le re-fetch post-commit de
`settle_invoice_handler` (`routes/invoices.rs:1192`) est **connu, sur `main`, et hors périmètre** :
ne le re-signale pas.

## Ce que tu rends

- **Les findings** : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), `fichier:ligne`, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que son état de
  départ est **atteignable par un chemin applicatif** ; s'il ne l'est pas, c'est au plus un LOW.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne compte pas comme passe, et sera repris à la main.

## Interdits

⛔ **N'écris aucun fichier et n'exécute aucune commande qui écrit** — dans le dépôt comme en base.
Nommément : `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`/`restore`, `sqlx migrate`,
`cargo test`, `cargo nextest`, `docker … restart`, et tout `mariadb` portant `INSERT`, `UPDATE`,
`DELETE`, `DROP` ou `CREATE`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext`, `cargo check`, `mariadb` en `SELECT`/`SHOW`.

---

## Lentille R — chasseuse de régressions, braquée sur la remédiation

**Ton objet est le seul commit `d13804ba`** — la remédiation de la passe 1 :

```sh
git show d13804ba --stat
git show d13804ba
```

Tu ne relis pas la story : tu relis **ce que la remédiation vient d'écrire**, en supposant qu'elle a
introduit le défaut suivant. C'est le motif mesuré du dépôt, et il est net : sur l'Epic 23, **sept
passes sur huit ont trouvé une régression du patch précédent et aucune un défaut de la conception
d'origine**.

1. **Le changement de signature.** `invoices::unvalidate` rend désormais
   `(Invoice, Vec<InvoiceLine>)`. Les lignes rendues sont lues **avant** l'`UPDATE` et rendues
   **après** — sont-elles celles de l'état final ? Un chemin peut-il modifier `invoice_lines` entre
   les deux ? La facture `after` et les `lines` peuvent-elles être incohérentes entre elles ? Tous
   les appelants ont-ils suivi (`grep -rn "unvalidate("`) ?
2. **Le handler.** La réponse construite a-t-elle exactement la même forme qu'avant — champs,
   ordre, `null` — pour un frontend qui la consomme déjà ? Un test l'établit-il, ou seulement une
   inspection partielle ?
3. **Les deux tests neufs** (`devalider_refuse_un_reglement_partiel_sans_paid_at`,
   `devalider_refuse_une_facture_creditee`) : leur montage construit-il vraiment l'état annoncé ?
   ⛔ **Le premier a déjà rougi une fois pour une raison de montage** (contrainte
   `chk_invoice_settlements_counterparty`), et un autre test de cette story a rougi pour la même
   famille de cause — cherche le montage qui ne déclenche pas ce qu'il croit. Que laissent-ils en
   base s'ils **paniquent au milieu** ? La base est **partagée** (`test_pool()`).
4. **Le nettoyage bancaire neuf.** La jointure qui retrouve `import_id` et `bank_account_id`
   rend-elle bien ce qu'on croit ? Que se passe-t-il si le test panique **avant** ? L'ordre des trois
   `DELETE` respecte-t-il les clés étrangères ?
5. **Le test HTTP neuf.** Ses deux moitiés sont-elles indépendantes — la seconde repart-elle d'un
   état propre après le `UPDATE … emailed_to = NULL` ? La `version` qu'elle réutilise est-elle
   encore la bonne après la première requête ? Le test passerait-il encore si la garde qu'il
   prétend couvrir était retirée ?
6. **Les corrections de documentation et de doc-comments** de ce commit disent-elles vrai, et
   **rien de plus** ? En particulier ce que `docs/api-external.md` affirme désormais du champ
   `details` : **vérifie chaque motif contre le code**, un par un.

## Lentille D — le périmètre complet, en un diff aplati

**Ton objet est le diff complet et unique** de la branche contre `main` :

```sh
git diff main...HEAD --stat
git diff main...HEAD -- crates/ docs/api-external.md
```

⚠️ **Prends ce diff aplati, et lui seul** — n'enchaîne pas les commits intermédiaires : les numéros
de ligne d'un second commit qui retouche les hunks d'un premier ne désignent pas le fichier final,
et cette confusion a déjà produit quatre findings `CRITICAL` faux dans ce dépôt. En cas de doute sur
une ligne, **lis le fichier**, ne déduis pas du diff.

1. **Les douze AC, un par un.** Pour chacun : quel code le tient, quel test l'établit — ou aucun.
   ⛔ **Deux passes ont déjà déclaré cette story couverte alors qu'elle ne l'était pas** : quatre
   manques trouvés par l'auteur, puis deux HIGH par la passe 1, tous sur des AC réputés tenus.
   Suppose qu'il en reste un.
2. **Les chemins d'écriture NON RÉSOLUS.** N'énumère pas les cas traités : **inventorie l'ensemble
   clos** des endroits qui font sortir une facture de `validated`, ou qui détachent ou suppriment
   son écriture — routes, repositories, `ON DELETE CASCADE`, import de sauvegarde, seed démo. Pour
   chacun : résolu, ou **angle mort assumé et écrit**. *(L'auteur a fait cet inventaire ; refais-le
   sans le lire, et compare à la fin.)*
3. **Les erreurs de bout en bout.** Pour chacun des neuf refus : le `DbError` construit, sa
   correspondance HTTP, son statut, son code, son `details`, sa clé i18n dans les **quatre** locales,
   et le **repli en dur** du code. Une seule incohérence dans cette chaîne est un finding.
4. **Les registres.** `audit_labels.rs` (présence et **ordre de tri**), `audit_route_registry.rs`
   (totaux en dur, partition, **et la ventilation du message**), `SITES_INDIRECTS`. ⛔ **Recompte
   depuis la source** — ne relis pas les valeurs, et ne fais pas confiance au fait qu'une passe
   précédente dise les avoir recomptées.
5. **Les comptes rendus.** Le story file et les messages de commit affirment des nombres : tests,
   fichiers, clés, mutations, totaux. **Recompte-les tous**, avec la commande, et déclare le
   **périmètre** de chaque décompte. ⚠️ Dans cette story, deux affirmations de compte rendu se sont
   déjà révélées fausses — c'est la zone la plus productive.
6. **Le manuel.** La story ne touche pas `docs/manual/`, mais **quelque chose y devient-il faux dès
   ce merge** ? Contrôle le **PDF**, pas seulement le `.tex`, et aplatis-le :
   `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`. Même question pour `README.md` et `website/`.
