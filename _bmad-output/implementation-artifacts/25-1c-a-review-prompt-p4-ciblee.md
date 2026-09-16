# Story 25-1c-a — prompt de la passe 4, CIBLÉE (chasseur de régression)

**Versionné le 2026-09-16**, § *La passe ciblée* du `CLAUDE.md`.

## Pourquoi encore une passe ciblée

- la **passe 3 n'a trouvé que des défauts de ma remédiation de la passe 2**, aucun de la conception ;
- le patch à relire ne touche **aucune ligne de code de production** — uniquement
  `crates/kesh-api/tests/audit_label_registry.rs`.

⚠️ **Ce second point est le critère de CLÔTURE de la boucle** : si cette passe ne rend rien au-dessus
de LOW, la boucle se ferme. Tant que le correctif touche la production, elle ne le peut pas.

## Modèle

**Opus** — la passe 3 était Sonnet.

## Les angles que je lui donne, et que je n'ai PAS éprouvés moi-même

1. `source_assainie` coupe par `take_while(|l| l.trim() != "#[cfg(test)]")` : que fait-elle d'un
   attribut **paramétré** (`#[cfg(test, …)]`), d'un `#[cfg(test)]` posé **au milieu** d'un fichier sur
   un simple helper, de plusieurs blocs, ou d'un fichier qui n'en a aucun ?
2. Les **chaînes brutes** `r"…"` et `r#"…"#` : mon scanner à état les traite comme des chaînes
   ordinaires, alors que `\"` n'y est **pas** un échappement. En existe-t-il dans les sept fichiers
   inventoriés ou dans `kesh-api/src` ?
3. Les **commentaires de bloc** `/* … */` : `strip_line_comments` ne les retire pas. Un code d'action
   cité dans un tel commentaire serait-il vu par le tamis ?
4. Le caractère d'échappement lui-même n'est pas poussé dans le contenu (`\\` → `\`, `\n` → `n`) :
   conséquence sur la comparaison des codes ?

## Prompt

> Tu es une lentille de revue ADVERSARIALE, **passe 4 CIBLÉE**, sur /home/gcorbaz/devel/kesh. Réponds
> en FRANÇAIS.
>
> PÉRIMÈTRE : **le seul commit `9ff606de`** — `git show 9ff606de`. Rien d'autre. C'est la remédiation
> de la passe 3 ; ton objet est **elle**, pas la conception de la story.
>
> Ce dépôt a mesuré que la sévérité se déplace vers ce qu'on vient d'écrire : sur les Epics 22 et 23,
> **sept passes sur huit** ont trouvé une régression du patch précédent. Les passes 1, 2 et 3 de cette
> story l'ont confirmé — chacune a trouvé un défaut introduit par la remédiation précédente.
>
> Ce que ce commit change, dans `crates/kesh-api/tests/audit_label_registry.rs` :
> 1. une fonction `source_assainie(brut)` remplace trois copies de
>    `strip_line_comments(brut.split("#[cfg(test)]").next()…)` ; elle coupe sur une **ligne entière**
>    (`take_while(|l| l.trim() != "#[cfg(test)]")`) et non sur une sous-chaîne ;
> 2. `litteraux_en_forme_de_code` passe d'un appariement `find('"')` naïf à un **suivi d'état**
>    (`dans_chaine` / `echappe`), et délègue le tamis à `retenir_si_code` ;
> 3. un décompte de commentaire passe de « six » à « sept » ; deux commentaires sont complétés.
>
> **Attaque précisément ceci** :
> - `source_assainie` : que fait-elle d'un `#[cfg(test)]` **paramétré**, **indenté**, posé au **milieu**
>   d'un fichier sur un helper plutôt qu'un `mod`, présent **plusieurs** fois, ou **absent** ? Y a-t-il
>   un fichier réel de `crates/*/src` que cette coupe ampute de code de PRODUCTION ? Vérifie-le, ne le
>   suppose pas.
> - le **scanner à état** : les **chaînes brutes** `r"…"` / `r#"…"#` sont traitées comme ordinaires,
>   or `\"` n'y est pas un échappement — en existe-t-il dans les sept fichiers inventoriés ou dans
>   `kesh-api/src` ? Quel littéral cela ferait-il apparaître ou disparaître à tort ?
> - les **commentaires de bloc** `/* … */`, que `strip_line_comments` ne retire pas : un code d'action
>   qui y serait cité tromperait-il le tamis ou le contrôle de l'inventaire ?
> - le caractère d'échappement n'est pas conservé (`\\` → `\`, `\n` → `n`) : conséquence réelle ?
> - `retenir_si_code` : l'ordre — `PAS_DES_CODES` consulté **avant** le tamis — change-t-il quelque
>   chose ? Le `.then(|| …)` est-il correct (évaluation paresseuse, emprunt) ?
> - la refonte a-t-elle **changé le comportement** des quatre tests préexistants ? Le balayage
>   voit-il désormais **plus** de source qu'avant, et cela fait-il apparaître des littéraux qui
>   n'étaient pas vus ?
> - les commentaires modifiés disent-ils vrai, mécanisme en main ?
>
> MÉTHODE IMPOSÉE :
> 1. ⛔ Pour tout finding CRITICAL ou HIGH, vérifie au sol (`grep -nF`, sortie collée) **ou** décris la
>    mutation exacte qui laisserait le test vert. Sans cela, irrecevable.
> 2. Classe CRITICAL / HIGH / MEDIUM / LOW avec fichier:ligne et correctif proposé.
> 3. Dis explicitement, en conclusion, si la remédiation que tu proposes toucherait du **code de
>    production** — c'est le critère de clôture de la boucle.
>
> INTERDICTIONS ABSOLUES : n'écris AUCUN fichier dans le dépôt (ni Write, ni Edit, ni redirection `>`)
> — /tmp seulement ; n'exécute AUCUN script mutant (`scripts/prepare-release.sh`,
> `scripts/regen-test-schema.sh`) ni AUCUNE commande `git` qui modifie l'état ; ne lance ni la suite
> complète ni les E2E. `cargo test -p kesh-api --test audit_label_registry` en lecture est autorisé.
>
> RENDU OBLIGATOIRE : termine par « AXES EXERCÉS » et « AXES NON EXERCÉS ». Un « 0 finding » sans ces
> deux listes ne compte pas comme passe.
