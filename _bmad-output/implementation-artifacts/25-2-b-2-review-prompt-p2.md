# Prompt — passe 2 de `bmad-code-review`, Story 25-2-b-2

*Versionné le 2026-09-23. Deux lentilles, en contexte frais, sur des modèles différents de la
passe 1 (Sonnet ×3) : **R** sur Opus, **D** sur Haiku 4.5.*

**Pourquoi deux, et pourquoi celles-là.** La passe 1 a rendu `0 HIGH, 2 MEDIUM`, et la remédiation
qui a suivi **touche du code de production** — le titre et le toast de la modale de suppression —
ainsi qu'un helper de montage partagé par des dizaines de tests. La boucle ne peut donc pas se
clore. Et le motif documenté au `CLAUDE.md` désigne l'endroit à relire en priorité : *la sévérité ne
stagne pas, elle se déplace vers ce qu'on vient d'écrire.* D'où une lentille **braquée sur la
remédiation** et une lentille **sur le périmètre complet**, plutôt que trois qui rejoueraient la
passe 1.

## Préambule commun

Tu es un **relecteur adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`, branche
`story/25-2-b-2-retrait-suppression-ecran-manuels`. La spec est
`_bmad-output/implementation-artifacts/25-2-b-2-retrait-suppression-ecran-manuels.md`.

⚠️ **La 25-2-b-1 est MERGÉE** (PR #448) : `unvalidate`, ses huit empêchements, sa route et le code
`INVOICE_MUST_BE_UNVALIDATED_FIRST` sont **hors périmètre**.

⛔ **Rien ne se croit sur parole** — ni la spec, ni les commentaires, ni le Change Log, ni les
messages de commit, **ni le rapport de la passe 1 qui y est résumé**. ⚠️ Et tiens ce soupçon
particulier : **la spec s'est déjà trompée une fois sur ce périmètre** (elle affirmait que la b-1
couvrait le motif « exercice clos » — elle avait vérifié que la contrepartie était *prescrite*, non
qu'elle *existait*), et **un compte rendu de cette story portait un total juste sous une ventilation
fausse**.

⛔ **Grep ground-truth obligatoire.** Tout finding `CRITICAL` ou `HIGH` affirmant **l'absence d'un
code attendu** ou **la présence d'un anti-pattern** se vérifie avant d'être rendu, par
`grep -nF "<chaîne exacte>" <fichier>` — le `-F` est **obligatoire**, le code est plein de
métacaractères. Pour un bloc, `grep -nFA 5`. Sans motif discriminant, lecture directe, et **cite
l'extrait lu**.

⚠️ **Ne conteste pas les arbitrages** : dévalider est ouvert au **Comptable**, effacer reste
**Administrateur** ; les clés API passent sur la dévalidation ; `emailed_at` est un refus sec ;
l'écran **assume le refus serveur** au lieu de rejouer la règle métier. Ne re-signale pas : les
issues **#449** et **#450** (ouvertes, hors périmètre) ; le **motif 8** (contre-passation) non
couvert sur le chemin de la dévalidation, **déclaré** au Dev Agent Record ; les deux écarts
repli/FTL **préexistants** `invoice-pdf-error-generic` et `invoices-download-pdf`.

## Ce que tu rends

- **Les findings** : sévérité, `fichier:ligne`, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour un scénario, montre que son état de départ est **atteignable par un chemin
  applicatif** ; sinon c'est au plus un LOW.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien et sera repris à la main.

## Interdits

⛔ **N'écris aucun fichier et n'exécute aucune commande qui écrit** — dans le dépôt comme en base.
Nommément : `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, **`make` dans `docs/manual/`**, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`/`restore`, `sqlx migrate`,
`cargo test`, `cargo nextest`, `npm run build`, `docker … restart`, et tout `mariadb` portant
`INSERT`, `UPDATE`, `DELETE`, `DROP` ou `CREATE`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext`, `cargo check`, `npm run check`, `mariadb` en
`SELECT`/`SHOW`.

---

## Lentille R — chasseuse de régressions, braquée sur la remédiation

**Ton objet est le seul commit `950cd628`** :

```sh
git show 950cd628 --stat
git show 950cd628
```

Tu ne relis pas la story : tu relis **ce que la remédiation vient d'écrire**, en supposant qu'elle a
introduit le défaut suivant.

1. ⛔ **`ensure_fiscal_year` rouvre désormais l'exercice qu'elle rend.** C'est un helper de montage
   appelé par des **dizaines** de tests. Quel test suppose, implicitement ou non, de trouver un
   exercice **clos** à cet appel ? Un test qui ferme l'exercice puis rappelle `create_and_validate`
   ou `force_validate` verrait-il sa fermeture annulée sous lui ? Cherche tous les appelants
   (`grep -n "ensure_fiscal_year\|force_validate\|create_and_validate"`) et dis pour chacun si le
   geste neuf peut changer ce qu'il mesure. **Un helper qui répare peut aussi effacer ce qu'un test
   voulait établir.**
2. **La condition `AND status <> 'Open'`** : est-elle correcte, et que fait-elle si `status` porte
   une autre valeur que `Open`/`Closed` ? Le `.unwrap()` sur l'`execute` est-il acceptable ici ?
3. **Les deux clés câblées** — `invoice-delete-confirm-title` sur le `Dialog.Title` et
   `invoice-deleted-success` sur le toast. Les replis correspondent-ils **mot pour mot** à leur
   fr-CH ? Le titre change de texte (« Supprimer la facture » → « Supprimer la facture ? ») :
   **un test, E2E ou unitaire, s'appuie-t-il sur l'ancien libellé** ? Cherche-le
   (`grep -rn "Supprimer la facture" frontend/`).
4. **Le repli d'`invoice-delete-confirm-body`** passe d'une question à une affirmation. Un test
   s'appuyait-il sur l'ancienne formulation ? Le texte affiché reste-t-il compréhensible dans la
   modale où il vit, sur les **deux** écrans ?
5. **La ventilation réécrite** du compteur `sitesTotal` : recompte-la **depuis la source**, terme à
   terme. `41 → 55` pour la fiche, `7 → 8` pour la liste, et la somme. La répartition annoncée
   entre clés neuves et clés réactivées est-elle exacte cette fois ?
6. **Ce que la remédiation n'a pas fait** : reste-t-il, sur ces deux écrans, un libellé français
   **codé en dur** qui aurait dû passer par `i18nMsg` — ou une clé du catalogue encore morte ?

## Lentille D — le périmètre complet, en un diff aplati

**Ton objet est le diff complet et unique** de la branche contre `main` :

```sh
git diff main...HEAD --stat
git diff main...HEAD -- crates/ frontend/ docs/ CHANGELOG.md README.md
```

⚠️ **Prends ce diff aplati, et lui seul** — n'enchaîne pas les commits intermédiaires : les numéros
de ligne d'un second commit qui retouche les hunks d'un premier ne désignent pas le fichier final,
et cette confusion a déjà produit quatre findings `CRITICAL` faux dans ce dépôt. En cas de doute sur
une ligne, **lis le fichier**, ne déduis pas du diff.

1. **Les onze AC, un par un.** Pour chacun : quel code le tient, quel test l'établit — ou aucun.
   ⛔ **Huit passes de validation ont déclaré cette story prête alors qu'un motif n'était couvert par
   aucun test.** Suppose qu'il reste un trou de cette nature.
2. **Les chemins de suppression, inventoriés et NON énumérés.** Combien de chemins, dans toute
   l'application, font disparaître une facture ou son écriture ? Routes, écrans, repositories,
   `ON DELETE CASCADE`. Pour chacun : traité, ou **angle mort assumé et écrit**. *(La spec en a
   raté un — l'écran de liste — pendant quatre passes.)*
3. **Le refus de bout en bout.** `INVOICE_MUST_BE_UNVALIDATED_FIRST` : le `DbError`, sa
   correspondance HTTP, son statut, son code, sa clé dans les **quatre** locales, son repli en dur,
   et ce que l'écran en affiche. Une seule incohérence dans cette chaîne est un finding.
4. **Les deux écrans.** Gardes de rôle sur le **bouton** et non sur le bloc ; modales distinctes ;
   aucun résidu du code retiré ; balises équilibrées. Un Comptable perd-il quelque chose qu'il
   avait ?
5. **Les comptes rendus.** Le story file et les messages de commit affirment des nombres —
   **recompte-les tous**, avec la commande, et déclare le **périmètre**. ⚠️ Deux affirmations de
   compte rendu se sont déjà révélées fausses dans cette famille de stories.
6. **La documentation.** Manuels (`.tex` **et** PDF aplati — `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`,
   l'apostrophe y est U+2019), `README.md`, `docs/api-external.md`, `CHANGELOG.md`. ⛔ Vérifie
   qu'**aucune ligne d'une section publiée du `CHANGELOG`** n'est touchée, et que
   `git diff main...HEAD -- crates/kesh-db/migrations/` est **vide** — sans quoi c'est un
   `CRITICAL` (**P8** : `sqlx` compare le checksum, le binaire ne boote plus).
