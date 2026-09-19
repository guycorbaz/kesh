# Story 25-1c-a — prompt de la passe 7, CIBLÉE (confirmation de clôture)

**Versionné le 2026-09-16**, § *La passe ciblée* du `CLAUDE.md`.

## État de la boucle

| passe | rendu | ce qu'elle a trouvé |
|---|---|---|
| 1 | 2 H, 5 M | la garde **affirmait au lieu de vérifier** — défaut de conception |
| 2 | 2 H, 4 M | le correctif fermait le *renommage*, pas l'*ajout* |
| 3 | **2 C**, 1 M | le tamis ignorait les échappements ; la coupe se fiait à la prose |
| 4 | 1 H, 2 M | la coupe fermait une variante et laissait l'autre |
| 5 | **1 C**, 1 M | l'appariement ne sortait pas sur un item sans bloc ; garde **verte par excès** |
| 6 | 3 H, 2 M | garde **verte par vacuité** (95 fichiers sur 99) ⇒ **SIMPLIFICATION** |
| 7 | — | la présente |

⚠️ **Septième passe sur les huit du budget.** Le code de production n'a plus été pris en défaut
**depuis la passe 2**, et la remédiation ne touche **aucune ligne de production depuis la passe 3** —
c'est le critère de clôture du `CLAUDE.md`, rempli. Il ne manque que la sévérité.

## Ce que la passe 6 a changé, et pourquoi c'est un changement de NATURE

Le balayage du dépôt a été **remplacé** par quatre entrées synthétiques écrites en dur. Motif mesuré :
une garde dont la sensibilité dépend de ce que le dépôt contient ce jour-là est verte par vacuité —
95 des 99 fichiers concernés ne l'exerçaient pas.

## Modèle

**Sonnet** — la passe 6 était Opus.

## Prompt

> Tu es une lentille de revue ADVERSARIALE, **passe 7 CIBLÉE**, sur /home/gcorbaz/devel/kesh. Réponds
> en FRANÇAIS.
>
> PÉRIMÈTRE : **le seul commit `25a7930d`** — `git show 25a7930d`. Rien d'autre.
>
> Les passes 1 à 6 ont **chacune** trouvé un défaut introduit par la remédiation précédente. Tu es là
> pour ce sens-là. ⚠️ **Septième passe sur huit**, et la boucle vise la clôture : le code de
> production n'est plus en cause depuis la passe 2.
>
> Ce que ce commit change, dans `crates/kesh-api/tests/audit_label_registry.rs` :
> 1. le test qui **balayait le dépôt** (`le_masquage_ne_retire_aucun_item_de_production`) et son helper
>    `est_item_de_production` sont **supprimés** ;
> 2. un test `le_masquage_se_verifie_sur_des_entrees_synthetiques` les remplace : quatre sources
>    **écrites en dur** — `mod` déclaré sans bloc, bloc ordinaire, bloc piégé par une accolade de
>    chaîne **et** un littéral de caractère, attribut sur une méthode ;
> 3. `accolades_hors_chaines` saute désormais les **littéraux de caractère** (`'x'`, `'\n'`) sans
>    confondre avec une **lifetime** (`&'static str`) ;
> 4. deux doc-comments sont corrigés (un « vérifié » qui était faux, un décompte de lecteurs retiré).
>
> **Attaque précisément ceci** :
> - ⛔ **La suppression a-t-elle fait PERDRE une couverture ?** Le balayage supprimé attrapait-il un
>   cas réel que les quatre entrées synthétiques n'exercent pas ? Nomme-le, ou établis qu'il n'y en a
>   pas.
> - ⛔ **Les quatre entrées synthétiques sont-elles SUFFISANTES ?** Quel arrangement Rust réel
>   n'exercent-elles pas, et qui casserait `source_assainie` ? (blocs imbriqués, plusieurs `#[cfg(test)]`
>   dans un fichier, attribut en dernière ligne, `mod tests` sans accolade sur la même ligne, macro
>   générant un bloc…)
> - ⛔ **Le saut de lifetime** : `&'static str`, `<'a>`, `'_`, `impl<'a> T for U<'a>` — le nouveau code
>   les traite-t-il correctement ? Une lifetime suivie d'une accolade sur la même ligne
>   (`impl<'a> Foo<'a> {`) fausse-t-elle le compte ? **Vérifie sur des lignes réelles du dépôt.**
> - Le littéral `'\''` (apostrophe échappée) et `'"'` (guillemet comme caractère) : que fait le
>   compteur ?
> - Les deux doc-comments corrigés disent-ils **enfin** vrai, mécanisme en main ?
> - Le comportement des neuf autres tests a-t-il changé ? Un ensemble bouge-t-il ?
>
> MÉTHODE IMPOSÉE :
> 1. ⛔ Pour tout finding CRITICAL ou HIGH, vérifie au sol (`grep -nF`, sortie collée) **ou** décris la
>    mutation exacte qui laisserait le test vert. Sans cela, irrecevable.
> 2. Classe CRITICAL / HIGH / MEDIUM / LOW avec fichier:ligne et correctif proposé.
> 3. **Conclus explicitement** : la boucle peut-elle être close ? La remédiation que tu proposes
>    toucherait-elle du **code de production** ?
>
> INTERDICTIONS ABSOLUES : n'écris AUCUN fichier dans le dépôt (ni Write, ni Edit, ni redirection `>`)
> — /tmp seulement ; n'exécute AUCUN script mutant (`scripts/prepare-release.sh`,
> `scripts/regen-test-schema.sh`) ni AUCUNE commande `git` qui modifie l'état ; ne lance ni la suite
> complète ni les E2E. `cargo test -p kesh-api --test audit_label_registry` en lecture est autorisé.
>
> RENDU OBLIGATOIRE : termine par « AXES EXERCÉS » et « AXES NON EXERCÉS ». Un « 0 finding » sans ces
> deux listes ne compte pas comme passe.
