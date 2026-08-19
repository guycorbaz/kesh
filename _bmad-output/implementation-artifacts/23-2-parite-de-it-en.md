# Story 23.2 : La parité — 57 clés, trois langues, et une allowlist qui disparaît

## Status

draft

Premier rollout de l'Epic 23, et le seul qui ferme **une issue entière** : [#283]. À son terme,
`crates/kesh-i18n/dette-parite-connue.txt` est **vide** et la garde de parité devient
**inconditionnelle** — plus aucune clé de `fr-CH` ne peut manquer aux trois autres locales.

## Story

**En tant qu'** utilisateur de Kesh en allemand, en italien ou en anglais,
**je veux** que les écrans de TVA, de paramètres de facturation et de validation de facture
s'affichent dans ma langue,
**afin de** ne pas recevoir du français sans le savoir — car le repli est **silencieux** : rien
n'indique que la traduction manque, le texte français s'affiche comme s'il était la traduction.

## Périmètre

**Dedans** : les **57 clés** listées dans `crates/kesh-i18n/dette-parite-connue.txt`, à écrire dans
`de-CH`, `it-CH` et `en-CH` ; le vidage de cette allowlist ; la fermeture de [#283].

**Dehors** : les 265 clés statiques de [#316] (rollouts 23-3 à 23-6) ; toute modification de
`fr-CH`, qui est la **source** ; toute modification du code applicatif.

⚠️ **Aucune clé ne peut être livrée partiellement.** La garde `parity_between_locales`
(`crates/kesh-i18n/src/loader.rs`) échoue dans les deux sens : sa clause (3) rejette une entrée
d'allowlist dès que la clé est traduite dans **au moins une** cible, tandis que sa clause (1) exige
sa présence dans **les trois**. Une clé traduite en `de-CH` seulement fait donc rougir le gate deux
fois. C'est voulu — c'est ce qui interdit la traduction partielle silencieuse.

## Dev Notes — ce qui est établi, pas supposé

### L'état de départ, mesuré

```
fr-CH : 57 / 57      de-CH : 0 / 57      it-CH : 0 / 57      en-CH : 0 / 57
```

Les 57 clés se répartissent en **trois domaines**, et ce découpage commande l'ordre de travail :

| domaine | clés | terminologie dominante |
|---|---:|---|
| `vat-rates-*` | 31 | taux de TVA, validité dans le temps, catégorie |
| `settings-invoicing-*` | 15 | comptes par défaut, journal, format de numérotation |
| `invoice-*` / `error-*` | 11 | validation, immuabilité, écriture comptable, exercice |

### ⚠️ Trois termes de la partie B du glossaire sont engagés — arbitrage requis AVANT le dev

Le glossaire (`docs/i18n-glossaire.md`) pose que les termes de sa partie B ne sont **pas attestés**
et qu'une fois tranchés ils remontent en partie A et deviennent contraignants. Trois le sont ici :

| terme | occurrences | proposition de la partie B |
|---|---|---|
| **immuable** | `invoice-validate-confirm-body`, `invoice-validate-success-body` | `unveränderlich` / `immutabile` / `immutable` |
| **validité** | `settings-vat-rates-link`, `vat-rates-subtitle` | `Gültigkeit` / `validità` / `validity` |
| **bascule** | `vat-rates-field-switch-date`, `vat-rates-change-hint` | ⚠️ **voir ci-dessous — la proposition est un CONTRESENS ici** |

⚠️ **`bascule` est un piège d'homonymie, et le glossaire induirait activement en erreur.** Sa ligne
de partie B lit « **bascule (interrupteur)** → `Umschalter` / `interruttore` / `toggle` », c'est-à-dire
l'**élément d'interface**. Or les deux occurrences visées ici désignent tout autre chose :

```
vat-rates-field-switch-date = Date de bascule
vat-rates-change-hint = L'ancien taux sera clôturé à la date de bascule, et le nouveau
                        taux prendra effet à cette date.
```

C'est la **date de changement de taux** — un basculement dans le temps, pas un interrupteur.
`Umschalter` y serait un contresens plein. C'est le même mode d'échec que le faux ami `individual`
trouvé en passe 1 de la 23-1b, **en pire** : là, le glossaire était muet ; ici, il donnerait une
réponse fausse à qui le consulte de bonne foi.

**Proposition** : traiter `bascule (changement de taux)` comme une **entrée distincte** de
`bascule (interrupteur)`, et rendre `date de bascule` par `Umstellungsdatum` / `data di
cambiamento` / `changeover date`. ⚠️ **Le glossaire doit porter les DEUX entrées**, faute de quoi le
piège se retendra au rollout suivant.

### Ce qui NE demande pas d'arbitrage : les termes comptables sont attestés

Les termes lourds de ce lot vivent déjà dans les quatre catalogues et se **relèvent** au lieu de
s'inventer — c'est la partie mécanique du travail, et la plus sûre :

| terme | occurrences dans `fr-CH` |
|---|---:|
| exercice | 44 |
| Statut | 8 |
| écriture comptable | 6 |
| Journal | 6 |
| taux de TVA | 3 |

⚠️ **Les relever, ce n'est pas les deviner** : pour chacun, retrouver la clé qui le porte et lire
les trois traductions cibles, exactement comme la 23-1b l'a fait pour ses vingt clés. Le moissonneur
ne sert **pas** ici — il moissonne des replis de code, or ces 57 clés ont déjà leur valeur `fr-CH`.

### Deux pièges de forme, hérités des cinq passes de revue de la 23-1b

1. **`settings-invoicing-format-help` porte des littéraux de chaîne Fluent** :
   `Placeholders : {"{"}YEAR{"}"}, {"{"}FY{"}"}, …`. Ce sont des **accolades échappées**, pas des
   placeables. Elles doivent être **recopiées à l'identique** dans les trois cibles ; les traduire
   ou les « corriger » casse le parse, et `loader.rs` propageant l'erreur sans tri, **une seule
   ligne cassée empêche le chargement de toute la locale**.
2. **`invoice-journal-entry-description` et trois autres portent de vrais placeables**
   (`{ $invoiceNumber }`, `{ $contactName }`). Les noms de variables ne se traduisent **jamais**.

### Registre — déjà mesuré, ne pas re-mesurer

`de-CH` en **Sie-Form**, `it-CH` au tutoiement, `en-CH` neutre (`docs/i18n-glossaire.md` § *Registre*).
Pas de `ß` en allemand suisse.

## Acceptance Criteria

1. **AC1** — les **57** clés de `dette-parite-connue.txt` existent dans `de-CH`, `it-CH` et `en-CH`,
   avec une valeur non vide et **différente du français** sauf justification écrite au cas par cas
   (« Journal », « CHF », « Total » sont légitimement identiques).
2. **AC2** — `crates/kesh-i18n/dette-parite-connue.txt` ne contient plus **aucune** entrée. Le
   fichier est conservé, vide et commenté, ou supprimé et son chargement adapté — **au choix de
   l'implémentation, mais le test doit rester fail-loud si le fichier disparaît par accident**.
3. **AC3** — `parity_between_locales` passe **sans allowlist**. ⚠️ **La borne anti-test-muet
   `>= 1200` est conservée** : un chargement cassé rendrait des ensembles vides et toute la
   comparaison serait verte.
4. **AC4** — `fr-CH` n'est **pas modifié**. Vérifiable : `git diff --stat` ne montre aucune ligne
   changée dans `locales/fr-CH/messages.ftl`.
5. **AC5** — les trois termes de partie B engagés sont **tranchés et promus en partie A** du
   glossaire, chacun avec la clé qui l'atteste. ⚠️ **`bascule` y figure en DEUX entrées distinctes**
   — l'interrupteur et le changement de taux —, faute de quoi le piège se retend au rollout suivant.
6. **AC6** — les littéraux de chaîne Fluent de `settings-invoicing-format-help` sont **identiques
   octet pour octet** dans les quatre locales. Vérifiable au `grep -F`.
7. **AC7** — aucun nom de variable de placeable n'est traduit : les `{ $… }` des quatre clés
   concernées sont identiques dans les quatre locales.
8. **AC8** — une **relecture consignée en tableau vérifiable au `grep`** des termes comptables
   relevés (exercice, écriture comptable, journal, taux de TVA, statut), sur le modèle du tableau
   `AC11-ter` de la 23-1b : terme, clé qui l'atteste, valeur dans les quatre locales.
9. **AC9** — gates complets verts, **exécutés et non recopiés** : `cargo fmt`, `clippy`,
   `cargo test -p kesh-i18n`, et côté frontend `check` / `lint-i18n-ownership` / `test:unit` /
   `build`. ⚠️ **Le gate backend complet est requis, pas seulement `-p kesh-i18n`** : cinq fichiers
   de `kesh-api` consomment les catalogues — `lib.rs`, `main.rs`, `errors.rs`, `routes/contacts.rs`
   et le test d'intégration `admin_full_export_e2e.rs` (relevé au `grep`, non supposé). Un ajout de
   **57 × 3 = 171** lignes aux catalogues peut donc faire bouger un test que cette story ne touche pas.
10. **AC10** — [#283] est **fermée par le mot-clé** dans le corps de la PR (`closes #283`), pas en
    prose. ⚠️ Précédent Story 16-3b : sept commits en `refs`, une PR disant « Ferme #151 » en
    toutes lettres, et l'issue restée ouverte après le merge sans le moindre signal.

## Tasks

- [ ] **T1 — Arbitrage terminologique** (AC5) : soumettre les trois termes de partie B, dont
      `bascule` et sa double entrée. **Bloquant** : ne pas commencer T2 avant.
- [ ] **T2 — Relevé des termes comptables** (AC8) : pour chacun, la clé qui l'atteste et ses quatre
      valeurs, consignés en tableau.
- [ ] **T3 — `vat-rates-*`** (31 clés) dans les trois cibles.
- [ ] **T4 — `settings-invoicing-*`** (15 clés), en recopiant à l'identique les littéraux échappés.
- [ ] **T5 — `invoice-*` / `error-*`** (11 clés), en préservant les placeables.
- [ ] **T6 — Vidage de l'allowlist** (AC2, AC3) et vérification que la garde échoue toujours pour
      une clé retirée d'une locale — **mutation, pas raisonnement**.
- [ ] **T7 — Glossaire** (AC5) : promotion des trois termes, double entrée pour `bascule`.
- [ ] **T8 — Gates complets** (AC9) et PR portant `closes #283` (AC10).

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-19 | création | spec initiale, terrain vérifié depuis la source (57/0/0/0, trois termes de partie B engagés, piège d'homonymie sur `bascule`) |
