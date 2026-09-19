# Story 25-1c-a — prompt de la passe 5, CIBLÉE (chasseur de régression)

**Versionné le 2026-09-16**, § *La passe ciblée* du `CLAUDE.md`.

## État de la boucle

| passe | rendu | ce qu'elle a trouvé |
|---|---|---|
| 1 | 2 H, 5 M | la garde **affirmait au lieu de vérifier** — défaut de conception |
| 2 | 2 H, 4 M | mon correctif fermait le *renommage*, pas l'*ajout* |
| 3 | **2 C**, 1 M | mon tamis ignorait les échappements ; ma coupe se fiait à la prose |
| 4 | 1 H, 2 M | ma coupe fermait une variante et laissait l'autre — 1017 lignes de production hors balayage |
| 5 | — | la présente |

⚠️ **Le code de production n'a plus été pris en défaut depuis la passe 2**, et le critère de clôture
— « la remédiation ne touche aucune ligne de production » — est rempli depuis la passe 3. La boucle
converge sur un seul fichier de **test**. Budget : 8 passes.

## Modèle

**Sonnet** — la passe 4 était Opus.

## Les angles que je donne, et que je n'ai PAS éprouvés

1. ⛔ **L'appariement ne se déclenche que s'il rencontre une accolade.** `source_assainie` n'arrête son
   saut que sur `ouvert && profondeur <= 0`. Un `#[cfg(test)]` posé sur un item **sans bloc** — `use`,
   `const`, `type`, un `mod` déclaré sans corps — ne produit aucune accolade : le saut consommerait
   **tout le reste du fichier**. Existe-t-il un tel site dans `crates/*/src` ?
2. ⛔ **Et le garde-fou que j'ai ajouté serait VERT PAR EXCÈS** dans ce cas : s'il ne reste plus rien,
   il ne reste plus d'attribut non plus. Le test `le_masquage_des_blocs_de_test_ne_laisse_aucun_attribut_derriere_lui`
   prouve-t-il quelque chose contre la sur-consommation ?
3. Le comptage `accolades_hors_chaines` ignore les **chaînes brutes** `r"…"`, les **littéraux de
   caractère** `'{'` / `'}'`, et les **commentaires de bloc** `/* { */`.
4. `l_inventaire_compte_ce_que_le_fichier_annonce` vérifie `len() == 7` mais **pas** que les mentions
   en toutes lettres du fichier disent « sept ».

## Prompt

> Tu es une lentille de revue ADVERSARIALE, **passe 5 CIBLÉE**, sur /home/gcorbaz/devel/kesh. Réponds
> en FRANÇAIS.
>
> PÉRIMÈTRE : **le seul commit `b7a06a29`** — `git show b7a06a29`. Rien d'autre.
>
> Les passes 1 à 4 ont chacune trouvé un défaut introduit par la remédiation précédente. Tu es là
> pour ce sens-là.
>
> Ce que ce commit change, dans `crates/kesh-api/tests/audit_label_registry.rs` :
> 1. `source_assainie` **masque** les blocs `#[cfg(test)]` par **appariement d'accolades** au lieu de
>    tronquer le fichier au premier attribut ;
> 2. une fonction `accolades_hors_chaines` compte les accolades en ignorant celles des chaînes ;
> 3. deux tests apparaissent : `le_masquage_des_blocs_de_test_ne_laisse_aucun_attribut_derriere_lui`
>    et `l_inventaire_compte_ce_que_le_fichier_annonce` ;
> 4. le scanner conserve désormais le caractère d'échappement ;
> 5. trois décomptes et deux commentaires sont corrigés.
>
> **Attaque en priorité ceci** :
> - ⛔ **La sur-consommation.** Le saut ne s'arrête que sur `ouvert && profondeur <= 0`. Un
>   `#[cfg(test)]` posé sur un item **sans accolade** (`use`, `const`, `type`, `mod x;`) ferait
>   consommer **tout le reste du fichier**. Cherche un tel site dans `crates/*/src` — et s'il n'y en a
>   pas, dis si un test l'empêcherait d'apparaître demain.
> - ⛔ **Le garde-fou serait-il vert par excès ?** Si le masquage consomme tout, aucun attribut ne
>   subsiste : le nouveau test passerait. Que prouve-t-il réellement ?
> - `accolades_hors_chaines` : chaînes brutes `r"…"` / `r#"…"#`, littéraux de caractère `'{'`,
>   commentaires de bloc `/* { */`, accolades de `format!` (`{}`), attributs `#[cfg(test)]` **indentés
>   dans un `impl`**.
> - Le masquage traite-t-il correctement **plusieurs** blocs de test dans un même fichier ? des blocs
>   **imbriqués** ?
> - `l_inventaire_compte_ce_que_le_fichier_annonce` : que ne vérifie-t-il pas ?
> - Le balayage voit-il désormais plus de source qu'avant ce commit, et cela révèle-t-il des littéraux
>   qui n'étaient pas vus ?
> - Les commentaires modifiés disent-ils vrai, mécanisme en main ?
>
> MÉTHODE IMPOSÉE :
> 1. ⛔ Pour tout finding CRITICAL ou HIGH, vérifie au sol (`grep -nF`, sortie collée) **ou** décris la
>    mutation exacte qui laisserait le test vert. Sans cela, irrecevable.
> 2. Classe CRITICAL / HIGH / MEDIUM / LOW avec fichier:ligne et correctif proposé.
> 3. Dis explicitement si la remédiation que tu proposes toucherait du **code de production**.
>
> INTERDICTIONS ABSOLUES : n'écris AUCUN fichier dans le dépôt (ni Write, ni Edit, ni redirection `>`)
> — /tmp seulement ; n'exécute AUCUN script mutant (`scripts/prepare-release.sh`,
> `scripts/regen-test-schema.sh`) ni AUCUNE commande `git` qui modifie l'état ; ne lance ni la suite
> complète ni les E2E. `cargo test -p kesh-api --test audit_label_registry` en lecture est autorisé.
>
> RENDU OBLIGATOIRE : termine par « AXES EXERCÉS » et « AXES NON EXERCÉS ». Un « 0 finding » sans ces
> deux listes ne compte pas comme passe.
