# Prompt — passe 4 **CIBLÉE** de `bmad-code-review`, Story 25-2-b-2

*Versionné le 2026-09-23. **Une** lentille (Opus ; P1 Sonnet ×3, P2 Opus + Haiku, P3 Sonnet), en
contexte frais, braquée sur **le seul commit `28d83b95`** — la remédiation de la passe 3.*

## Pourquoi cette passe

La boucle converge : `2M → 2M/3L → 1M/1L`, aucun `HIGH` ni `CRITICAL` depuis le début. Mais la
remédiation de la passe 3 **touche encore du code de production** — la modale de validation, quatre
catalogues de traduction — et surtout elle s'attaque au motif qui a déjà récidivé **quatre fois** sur
cette seule story.

⛔ **Le compteur i18n a été corrigé quatre fois** — 1651 → 1653 → 1655 → 1660 → 1665 — et **chaque
correction a laissé un reste** :

1. une ventilation fausse sous un total juste ;
2. deux clés câblées sur **un seul** des deux écrans ;
3. les **boutons**, invisibles au contrôle repli/FTL ;
4. la modale de **validation**, invisible aux trois contrôles précédents parce qu'ils partaient tous
   d'une **liste de clés**.

**Ta question est simple : y a-t-il un cinquième ?** Et si tu n'en trouves pas, tu dois l'établir,
pas le supposer.

## Ce que tu fais

```sh
git show 28d83b95 --stat
git show 28d83b95
```

1. ⛔ **Le contrôle qui a marché, refait à ta façon.** Le patch prétend inventorier l'**ensemble
   clos** des textes visibles non traduits des deux écrans — dix sites, tous antérieurs.
   **Reconstruis cet inventaire indépendamment**, avec ta propre méthode, et compare. Cherche large :
   contenu de balise, `aria-label`, `title`, `placeholder`, `alt`, `value` de bouton, arguments de
   `notifySuccess`/`notifyError`, affectations à une variable d'erreur, chaînes passées à un
   composant. ⚠️ **Un inventaire faux est pire qu'absent** : il certifie.
2. **Les quatre sites câblés.** Corps, deux boutons, toast. Les replis disent-ils **mot pour mot**
   leur fr-CH ? La clé neuve `invoice-validate-success-no-number` est-elle dans les **quatre**
   locales, et sa traduction est-elle juste dans chacune ? L'argument `{ $invoiceNumber }` est-il
   passé sous le bon nom ?
3. **La branche conditionnelle du toast** : les deux chemins sont-ils atteignables, et le second
   (sans numéro) est-il vraiment défensif — ou bien un état réel l'emprunte-t-il ? *Une facture
   dévalidée puis revalidée garde son numéro : le second chemin reste-t-il mort ?*
4. **La ventilation du compteur**, recomptée terme à terme depuis la source, aux **trois** bornes
   (début de story, avant ce commit, courant). Et **la classe** de chaque site neuf : clé neuve, clé
   réactivée, ou clé déjà en service. *C'est la classe qui s'est trompée deux fois, jamais la somme.*
5. **Ce que le commit affirme.** Il déclare que « tous » les dix sites restants sont antérieurs et
   qu'« aucun n'apparaît dans son diff ». Vérifie-le site par site. Il affirme aussi qu'« aucun gate
   ne les voit » : est-ce exact pour les deux gardes nommées ?
6. **Le reste du diff de la branche** n'est PAS ton objet — sauf si un défaut du commit `28d83b95`
   y a des conséquences.

## Ce que tu rends

- **Les findings** : sévérité, `fichier:ligne`, **la commande ou l'extrait qui l'établit**, le
  correctif. Un texte visible non traduit **dans le périmètre** est MEDIUM ; antérieur, LOW. Une
  affirmation de compte rendu plus large que le fait vérifié est MEDIUM. Un inventaire incomplet
  qui se présente comme clos est MEDIUM.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien.

## Interdits

⛔ **N'écris aucun fichier et n'exécute aucune commande qui écrit** — dans le dépôt comme en base.
Nommément : `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`/`restore`, `sqlx migrate`,
`cargo test`, `cargo nextest`, `npm run build`, `docker … restart`, et tout `mariadb` portant
`INSERT`, `UPDATE`, `DELETE`, `DROP` ou `CREATE`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `pdftotext`, `cargo check`, `npm run check`, `mariadb` en `SELECT`/`SHOW`.

⚠️ **Ne re-signale pas** : les issues **#449** et **#450** ; le **motif 8** non couvert ; les écarts
repli/FTL **préexistants** `invoice-pdf-error-generic` et `invoices-download-pdf` ; et le fait que
les deux gardes i18n ne voient pas cette classe de défaut — c'est écrit et porté à la rétrospective.
Les **dix sites inventoriés** ne sont pas à re-signaler un par un ; en revanche, **un onzième que
l'inventaire aurait manqué est un finding**.
