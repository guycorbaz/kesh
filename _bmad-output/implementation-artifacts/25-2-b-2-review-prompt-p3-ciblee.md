# Prompt — passe 3 **CIBLÉE** de `bmad-code-review`, Story 25-2-b-2

*Versionné le 2026-09-23. **Une** lentille (Sonnet ; P1 était Sonnet ×3, P2 Opus + Haiku), en
contexte frais, braquée sur **le seul commit `599ccfb0`** — la remédiation de la passe 2.*

## Pourquoi cette passe, et pourquoi elle est ciblée

La boucle converge en sévérité — `2 MEDIUM` en passe 1, `2 MEDIUM + 3 LOW` en passe 2, aucun HIGH
depuis le début —, mais la remédiation de la passe 2 **touche encore du code de production** : les
deux écrans, quatre catalogues de traduction, et un helper de montage partagé. Elle ne peut donc pas
clore la boucle.

⛔ **Et le motif qui te concerne est mesuré sur cette story même : trois récidives d'affilée du
même symptôme.** Le compteur i18n a été corrigé **trois fois** — 1651 → 1653 → 1655 → 1660 — et
chaque correction a laissé un reste :

1. une ventilation fausse sous un total juste ;
2. deux clés câblées sur **un seul** des deux écrans, alors que la fiche prévenait qu'« un symptôme
   se grepe sur les DEUX écrans » ;
3. les **boutons**, que le contrôle repli/FTL ne voyait pas — *il ne compare que ce qui passe déjà
   par `i18nMsg`, et un détecteur ne trouve rien là où l'appel n'existe pas encore*.

**Ton travail est de trouver le quatrième reste**, s'il existe.

## Ce que tu fais, exactement

```sh
git show 599ccfb0 --stat
git show 599ccfb0
```

1. ⛔ **La propagation, cette fois jusqu'au bout.** Ne cherche pas les écarts de traduction :
   cherche **tout texte visible de l'utilisateur qui ne passe pas par `i18nMsg`** dans les deux
   écrans de factures — contenu de balise, attribut `aria-label`, `title`, `placeholder`, `alt`,
   argument de `notifySuccess`/`notifyError`/`confirm`. Liste-les tous, puis dis pour chacun s'il
   est dans le périmètre de cette story ou antérieur. *Deux sont déjà déclarés hors périmètre —
   « Valider » et « Modifier » de la barre d'action ; tout le reste est à établir.*
2. **La clé neuve `invoice-delete-confirm-body-context`** : présente dans les **quatre** locales ?
   Ses deux variables `{ $date }` et `{ $contact }` sont-elles **réellement passées** au site
   d'appel, et sous le même nom ? Le repli en dur dit-il la même chose que le fr-CH, mot pour mot ?
   Que s'affiche-t-il si `deleteTarget` est `null` ?
3. **La réparation bavarde d'`ensure_fiscal_year`** : le message est-il exact dans ce qu'il affirme
   (« des écritures et une ligne de compteur **peuvent** subsister ») ? `rows_affected() == 1`
   est-il le bon test ? Le `ORDER BY id` ajouté change-t-il ce que renvoie le helper dans un cas
   existant ?
4. **Le doc-comment reformulé en « constat daté »** : ce qu'il affirme des appelants d'aujourd'hui
   est-il vrai ? Recompte les fermetures d'exercice du module et leur position relative aux appels
   de montage.
5. ⛔ **La ventilation du compteur, recomptée terme à terme depuis la source.** `41 → 57` pour la
   fiche, `7 → 13` pour la liste, la somme, et **la classe de chaque site** — neuve, réactivée, ou
   déjà en service. *C'est la classe qui s'est trompée deux fois, jamais la somme.*
6. **Ce que les messages de commit affirment.** Trois d'entre eux déclarent une propagation close
   (« plus aucun toast codé en dur », « zéro écart », « contrôle rejoué sur les deux écrans »).
   Chacun est-il vrai **dans sa portée exacte** ? Une affirmation vraie mais plus large que ce qui
   a été vérifié est un finding : c'est ce qui a permis la troisième récidive.

## Ce que tu rends

- **Les findings** : sévérité, `fichier:ligne`, **la commande ou l'extrait qui l'établit**, le
  correctif. Un texte visible de l'utilisateur non traduit est au moins LOW ; dans le périmètre de
  la story, MEDIUM. Une affirmation de compte rendu plus large que le fait vérifié est MEDIUM.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien : il sera repris à la main.

## Interdits

⛔ **N'écris aucun fichier et n'exécute aucune commande qui écrit** — dans le dépôt comme en base.
Nommément : `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`/`restore`, `sqlx migrate`,
`cargo test`, `cargo nextest`, `npm run build`, `docker … restart`, et tout `mariadb` portant
`INSERT`, `UPDATE`, `DELETE`, `DROP` ou `CREATE`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `pdftotext`, `cargo check`, `npm run check`, `mariadb` en `SELECT`/`SHOW`.

⚠️ **Ne re-signale pas** : les issues **#449** et **#450** ; le **motif 8** (contre-passation) non
couvert, déclaré ; les écarts repli/FTL **préexistants** `invoice-pdf-error-generic` et
`invoices-download-pdf` ; « Valider » et « Modifier » de la barre d'action, déclarés hors périmètre ;
et le fait que `i18n-libelle-en-dur` ne voie pas cette classe de défaut — c'est écrit.
