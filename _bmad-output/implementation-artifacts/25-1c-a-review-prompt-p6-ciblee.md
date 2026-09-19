# Story 25-1c-a — prompt de la passe 6, CIBLÉE (chasseur de régression)

**Versionné le 2026-09-16**, § *La passe ciblée* du `CLAUDE.md`.

## État de la boucle — et la question qu'elle pose

| passe | rendu | ce qu'elle a trouvé |
|---|---|---|
| 1 | 2 H, 5 M | la garde **affirmait au lieu de vérifier** — défaut de conception |
| 2 | 2 H, 4 M | mon correctif fermait le *renommage*, pas l'*ajout* |
| 3 | **2 C**, 1 M | le tamis ignorait les échappements ; la coupe se fiait à la prose |
| 4 | 1 H, 2 M | la coupe fermait une variante et laissait l'autre |
| 5 | **1 C**, 1 M | l'appariement ne sortait pas sur un item sans bloc ; le garde-fou était **vert par excès** |
| 6 | — | la présente |

⚠️ **Sixième passe sur les huit du budget.** La sévérité **ne décroît pas**, le code de production n'a
plus été pris en défaut **depuis la passe 2**, et les cinq passes se sont jouées sur **un seul fichier
de test**. Un arbitrage de conduite est porté au Project Lead : *ce détecteur vaut-il ce qu'il coûte ?*
**Ma recommandation, si cette passe trouve encore : SIMPLIFIER la garde plutôt que la raffiner.**

## Modèle

**Opus** — malgré la répétition avec la passe 4 : l'axe est une analyse lexicale fine, et la règle dit
« LLM différent **si possible** ». La profondeur prime ici sur la rotation.

## Les angles que je donne, et que je n'ai PAS éprouvés

1. ⛔ **`est_item_de_production` énumère des préfixes** — `pub fn `, `fn `, `struct `… — et **omet
   `pub(crate) fn`, `pub(super) fn`, `unsafe fn`, `pub(crate) struct`**. Or ce dépôt en emploie (j'en
   ai écrit dans cette story même). Ces items ne sont donc **pas protégés** par la garde symétrique.
   *C'est une énumération de formes qui marchent, exactement ce que D4-ter proscrit.*
2. `assainie.contains(ligne.trim_end())` : si la **même ligne** existe deux fois — une en production,
   une indentée dans un bloc de test — le `contains` peut être satisfait par la mauvaise. Faux négatif.
3. La **sortie 2** (`!ouvert && profondeur == 0 && ligne se termine par ';'`) : quelle ligne réelle
   pourrait la déclencher **trop tôt** ?

## Prompt

> Tu es une lentille de revue ADVERSARIALE, **passe 6 CIBLÉE**, sur /home/gcorbaz/devel/kesh. Réponds
> en FRANÇAIS.
>
> PÉRIMÈTRE : **le seul commit `75852e33`** — `git show 75852e33`. Rien d'autre.
>
> Les passes 1 à 5 ont **chacune** trouvé un défaut introduit par la remédiation précédente. Tu es là
> pour ce sens-là. ⚠️ Sixième passe sur huit : si tu trouves encore, dis explicitement si le défaut
> justifie un correctif **ou si la garde devrait être SIMPLIFIÉE** — elle protège un fichier de test,
> et le code de production n'a plus été pris en défaut depuis la passe 2.
>
> Ce que ce commit change, dans `crates/kesh-api/tests/audit_label_registry.rs` :
> 1. `source_assainie` gagne une **seconde sortie de boucle** : l'item gardé se termine sur son
>    point-virgule sans avoir ouvert de bloc ;
> 2. un test `le_masquage_ne_retire_aucun_item_de_production` et un helper `est_item_de_production`
>    apparaissent — la garde **symétrique** de celle qui vérifie l'absence d'attribut résiduel ;
> 3. le message de `l_inventaire_compte_ce_que_le_fichier_annonce` ne chiffre plus les occurrences ;
> 4. les limites d'`accolades_hors_chaines` sont documentées.
>
> **Attaque en priorité ceci** :
> - ⛔ **`est_item_de_production` est une ÉNUMÉRATION DE PRÉFIXES.** Lesquels manquent ?
>   (`pub(crate) fn`, `pub(super) fn`, `unsafe fn`, `pub(crate) struct`, `extern`, `type `, `union `,
>   `macro_rules!`…) Cherche dans `crates/*/src` des items réels qu'elle **ne reconnaît pas** et qui
>   suivent un `#[cfg(test)]` — ils ne sont alors protégés par rien. Quel est le mode d'échec exact ?
> - `assainie.contains(ligne.trim_end())` : une ligne dupliquée (production + test indenté) peut-elle
>   satisfaire le `contains` à tort ? Trouve un cas réel ou montre qu'il n'y en a pas.
> - La **sortie 2** : quelle ligne réelle de `crates/*/src` la déclencherait **trop tôt**, coupant un
>   bloc de test avant sa fin ? (attributs multiples, `#[cfg(test)]` suivi d'un `use` puis d'un `mod`,
>   signature multi-lignes, macro…)
> - Le masquage traite-t-il un `#[cfg(test)]` en **toute dernière ligne** du fichier ? un fichier qui
>   n'en contient qu'un, sans rien après ?
> - Les commentaires et la documentation des limites disent-ils vrai, **mécanisme en main** ?
> - Le balayage voit-il plus ou moins de source qu'avant ce commit ? Cela change-t-il un ensemble ?
>
> MÉTHODE IMPOSÉE :
> 1. ⛔ Pour tout finding CRITICAL ou HIGH, vérifie au sol (`grep -nF`, sortie collée) **ou** décris la
>    mutation exacte qui laisserait le test vert. Sans cela, irrecevable.
> 2. Classe CRITICAL / HIGH / MEDIUM / LOW avec fichier:ligne et correctif proposé.
> 3. Dis explicitement si la remédiation toucherait du **code de production**, et si tu recommandes de
>    **simplifier** la garde plutôt que de la corriger.
>
> INTERDICTIONS ABSOLUES : n'écris AUCUN fichier dans le dépôt (ni Write, ni Edit, ni redirection `>`)
> — /tmp seulement ; n'exécute AUCUN script mutant (`scripts/prepare-release.sh`,
> `scripts/regen-test-schema.sh`) ni AUCUNE commande `git` qui modifie l'état ; ne lance ni la suite
> complète ni les E2E. `cargo test -p kesh-api --test audit_label_registry` en lecture est autorisé.
>
> RENDU OBLIGATOIRE : termine par « AXES EXERCÉS » et « AXES NON EXERCÉS ». Un « 0 finding » sans ces
> deux listes ne compte pas comme passe.
