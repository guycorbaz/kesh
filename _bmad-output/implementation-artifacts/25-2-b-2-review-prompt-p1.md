# Prompt — passe 1 de `bmad-code-review`, Story 25-2-b-2

*Versionné le 2026-09-23. Trois lentilles en contexte frais (Sonnet), orthogonales à l'auteur de
l'implémentation (Opus 5). Chaque lentille reçoit ce préambule et **son seul** bloc d'axes.*

## Préambule commun

Tu es un **relecteur adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`, branche
`story/25-2-b-2-retrait-suppression-ecran-manuels`. Le périmètre est le diff **`main...HEAD`** :

```sh
git diff main...HEAD --stat
git diff main...HEAD -- crates/ frontend/ docs/ CHANGELOG.md README.md
```

La spec est `_bmad-output/implementation-artifacts/25-2-b-2-retrait-suppression-ecran-manuels.md` ;
la fiche mère `25-2-b-devalidation-facture.md` reste la source des faits établis et ne se rediscute
pas. ⚠️ **La 25-2-b-1 est MERGÉE** (PR #448) : elle a posé `unvalidate`, ses huit empêchements, la
route et le code `INVOICE_MUST_BE_UNVALIDATED_FIRST` — tout cela est **hors périmètre**.

⛔ **Rien ne se croit sur parole** — ni la spec, ni les commentaires, ni le Dev Agent Record, ni les
messages de commit. ⚠️ **Et la spec elle-même s'est déjà trompée sur ce périmètre** : elle affirmait
que la 25-2-b-1 couvrait le motif « exercice clos », ce qui était faux — *elle avait vérifié que la
contrepartie était prescrite, non qu'elle existait*. Tiens le même soupçon pour le reste.

⛔ **Grep ground-truth obligatoire.** Tout finding `CRITICAL` ou `HIGH` affirmant **l'absence d'un
code attendu** ou **la présence d'un anti-pattern** se vérifie avant d'être rendu, par
`grep -nF "<chaîne exacte>" <fichier>` — le `-F` est obligatoire. Pour un bloc, `grep -nFA 5`. Pour
un flux cross-fonction, lecture directe, et **cite l'extrait lu**.

⚠️ **Ne conteste pas les arbitrages** : dévalider est ouvert au **Comptable**, effacer reste
**Administrateur** ; les clés API passent sur la dévalidation ; `emailed_at` est un refus sec. Et ne
re-signale pas les issues **#449** (`api-external.md` annonce une faille corrigée comme ouverte) ni
**#450** (`settle_invoice_handler` et son re-fetch post-commit) : ouvertes, connues, hors périmètre.

## Ce que tu rends

- **Les findings** : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), `fichier:ligne`, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que son état de
  départ est **atteignable par un chemin applicatif**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne compte pas comme passe et sera repris à la main.

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

## Lentille A — le dépôt, le refus, et les huit sites de test

1. **Le retrait est-il complet ET sûr ?** Le bras `validated` de `invoices::delete`, et surtout le
   **bloc hors `match`** qui appelait `delete_in_tx(…, false)`. Reste-t-il, dans tout le dépôt, un
   appelant de production passant `false` **autre que** `unvalidate` (`grep -rn
   "delete_in_tx" crates/`) ? Un chemin peut-il encore faire disparaître l'écriture d'une facture
   par `delete` ?
2. **Le refus.** `INVOICE_MUST_BE_UNVALIDATED_FIRST` est-il rendu **partout où il doit l'être**, et
   nulle part où il ne doit pas ? Le bras fourre-tout `Some(inv) =>` peut-il encore attraper une
   facture validée ? Que rend `delete` sur une facture `cancelled`, ou sur un brouillon portant un
   `journal_entry_id` non nul (état atteignable ?) ? Le `rollback` est-il fait sur ce chemin ?
3. **Les huit sites de test, un par un.** Pour chacun des quatre **supprimés**, vérifie
   **nommément** que sa propriété est reprise par un test existant — cite le test et son assertion.
   ⛔ **C'est exactement là que la spec s'est trompée** : refais le travail, ne le relis pas. Les
   deux **retargetés** : leur nom décrit-il ce qu'ils mesurent ? Le **réécrit** exerce-t-il bien le
   verrou de période *par la dévalidation* ?
4. **Le test neuf `devalider_refuse_un_exercice_clos`** : tranche-t-il ? Son montage construit-il
   l'état annoncé, ou un autre empêchement parle-t-il avant ? Que laisse-t-il en base **s'il panique
   au milieu** — la base des tests de dépôt est **partagée** (KF-039, #310) ?
5. **Ce qui n'est couvert nulle part.** Inventorie les **huit motifs** et dis, pour chacun, quel
   test l'assert **sur le chemin de la dévalidation**. Le Dev Agent Record en déclare un non
   couvert ; vérifie qu'il n'y en a pas d'autres.

## Lentille B — les deux écrans, l'i18n et les registres

1. **Les deux gardes de rôle.** Sur la fiche **et** sur la liste : la garde porte-t-elle sur le
   **bouton** et non sur le `{#if}` qui enveloppe aussi d'autres actions ? Un Comptable garde-t-il
   *Valider* et *Modifier* sur un brouillon ? Voit-il « Dévalider » sur une facture validée ?
2. **La modale de dévalidation.** Est-elle bien **distincte** de celle de suppression — état,
   titre, corps, bouton, appel ? Un chemin peut-il encore ouvrir la modale de suppression depuis une
   facture validée ? Que se passe-t-il si l'appel échoue : la modale reste-t-elle ouverte avec le
   motif, et l'écran reste-t-il cohérent ?
3. **Le résidu retiré.** `deleteConfirmText`, sa remise à zéro, la condition du bouton, l'import
   `Input` : tout est-il parti, et **rien de plus** ? Le `{:else}` et le `{/if}` sont-ils
   équilibrés ? ⚠️ La condition du corps de la modale **change** (statut → présence d'un numéro) :
   est-ce correct pour un brouillon **jamais validé** ?
4. **L'i18n.** Les **7 clés neuves** dans les **4** locales, l'argument `{ $number }` réellement
   passé, les replis en dur cohérents avec les locales. Les **2 messages réécrits** disent-ils vrai
   maintenant — et leurs replis Rust aussi ? Une clé neuve est-elle jamais lue ? Une clé lue
   manque-t-elle ?
5. **Les registres, RECOMPTÉS depuis la source.** `i18n-keys.test.ts` (`sitesTotal`, et sa
   **ventilation** écrite), `i18n-libelle-en-dur.test.ts` (candidates et partition, et la
   déclaration **nommée**), `audit_route_registry.rs`. ⛔ Ne relis pas les valeurs : recompte-les,
   avec la commande.

## Lentille C — les manuels, la documentation, et les comptes rendus

1. **Le manuel utilisateur.** La section refondue dit-elle vrai, **et complètement** ? Les huit
   refus y sont-ils, et correspondent-ils au code ? Les cinq autres lignes recalées (verrou de
   période, « un chemin qui creuse », « pas de numéro définitif », « strictement séquentielle »,
   le renvoi de l'avoir) sont-elles justes ? **Reste-t-il, dans tout le manuel, une phrase qui
   décrit le chemin retiré ou qui ignore la dévalidation ?**
2. **Le manuel d'administration.** Les quatre sites. ⚠️ En particulier : « ni modifiable ni
   supprimable » — la nuance ajoutée est-elle **exacte** ? Et l'affirmation neuve sur ce qu'une clé
   API peut faire : vérifie-la **contre le code**, pas contre la fiche.
3. ⛔ **Les PDF, aplatis.** `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`. Sont-ils à jour du
   `.tex` ? ⚠️ `pdftotext` rend l'apostrophe en `’` (U+2019) : greper une phrase telle qu'elle
   s'écrit dans la source rend **zéro** et fait conclure à tort. Contrôle aussi la **brochure**.
4. **`CHANGELOG.md`.** ⛔ **Aucune ligne d'une section PUBLIÉE ne doit être touchée** — vérifie-le
   par `git diff --numstat` et par lecture. Le « 123 libellés » de la `[0.12.0]` est-il intact ? Le
   **124** de la section neuve se recompte-t-il depuis `audit_labels.rs` ? Le titre de section
   est-il au format **exact** qu'attend `scripts/prepare-release.sh` ?
5. **`README.md` et `docs/api-external.md`** : justes, et cohérents entre eux et avec les manuels ?
6. **Les comptes rendus.** ⛔ **Recompte tout ce que le story file et les messages de commit
   affirment** — tests, clés, sites, compteurs —, avec la commande, et déclare le **périmètre**.
   ⚠️ Sur la story sœur, **deux affirmations de compte rendu se sont révélées fausses** : c'est la
   zone la plus productive du dépôt.
7. ⛔ **Le site sous P8.** `crates/kesh-db/migrations/20260715000001_invoice_reminders.sql:14` porte
   une affirmation périmée qu'il est **interdit** de corriger : `sqlx` compare le checksum et le
   binaire ne boote plus. Vérifie que `git diff main...HEAD -- crates/kesh-db/migrations/` est
   **vide**. S'il ne l'est pas, c'est un `CRITICAL`.
