# Story 63-1 — prompt de la revue de code P3 (Haiku, passe CIBLÉE, contexte frais)

Tu conduis la **passe ciblée P3** de la story 63-1 de Kesh, avec le modèle **Haiku** (la P2 était Sonnet, la P1 Opus). Une seule lentille, sur le
**seul commit de la dernière remédiation** : `<SHA à renseigner par l'orchestrateur : le commit qui corrige M1′ de la P2>`.
Aide-toi d'un diff unique aplati (`git diff <SHA>^..<SHA>`), non d'une lecture commit par commit. Contexte : Kesh est un logiciel de comptabilité
suisse, tenu en production par Guy ; le manuel est lu par un utilisateur suisse qui y trouve ce qu'il doit faire auprès de l'AFC et de sa fiduciaire.

## La lentille unique : « le manuel dit-il ce que fait le code, et rien de plus ? »

La P2 (rapport `$GATE_LOGS/e63/63-1-revue-p2.md`) a rendu **un MEDIUM (M1′)** : les phrases ajoutées au manuel par la remédiation de M1 conseillent un
recul de verrou suivi d'un **décompte rectificatif auprès de l'AFC** à deux endroits où l'annulation **ne change aucun décompte de TVA** :
- `docs/manual/fr/user-manual.tex`, § *Enregistrer et annuler un règlement* (liste des motifs, puce « le règlement est dans une période verrouillée ») :
  « l'annulation en change les totaux (TVA comprise) … l'écart se corrige auprès de l'AFC » ;
- `docs/manual/fr/user-manual.tex`, § règlement fournisseur (fin du paragraphe « le paiement est dans une période verrouillée ») :
  « le recul du verrou en change les totaux, et Kesh ne garde pas le décompte déposé : l'écart se corrige auprès de l'AFC ».

Prémisse à **relire au code** (ne pas la croire) : `crates/kesh-report/src/vat_report.rs` (TVA due = lignes des factures **validées** datées dans la période ;
TVA récupérable = solde du compte d'impôt préalable ; soldes = diminutions de contre-prestation). Une écriture de règlement (banque contre débiteurs, ou
créanciers contre banque) ne porte aucune ligne de TVA : annulée, elle change les totaux du bilan de la période, **non** le décompte de TVA. Seuls l'**achat**
(impôt préalable, charge) et le **solde** (escompte, perte, TVA comprise) le changent.

Ce que la remédiation devait faire (à vérifier phrase par phrase) :
1. aux deux endroits ci-dessus, **retirer** l'indication « TVA comprise » et le renvoi à l'AFC / au décompte rectificatif, ou les remplacer par ce qui est vrai
   (les totaux du **bilan** de la période changent ; la TVA ne change pas) ; **ne pas** le retirer des endroits justes : `admin-manual.tex` (verrou, facture
   fournisseur), `user-manual.tex` § *Annuler une facture fournisseur* et § décompte de TVA (« Le solde annulé »), et la phrase de la ligne ~872 ;
2. ne **changer aucune règle**, ne rien promettre que Kesh ne fasse pas (Kesh ne garde pas le décompte déposé : vrai, à conserver là où la TVA change) ;
3. LOW de la P2 traités dans le même commit, s'il y en a (terme « décompte rectificatif » ; `admin-manual.tex` § « La date du jour est celle de Zurich » :
   « contre-passations et annulations » ; confirmations FTL des règlements) : relire chacun au code et à la source.

## Axes (déclare, à la fin, ceux que tu as exercés et ceux que tu n'as pas exercés)

1. Chaque phrase neuve ou modifiée contre le code (`vat_report.rs`, `settlement_cancellation.rs`) : vraie ? complète ? sans promesse ?
2. **Propagation du symptôme** : `git grep -nF "TVA comprise"`, `git grep -nF "décompte rectificatif"`, `git grep -nF "auprès de l'AFC"` dans `docs/manual/fr/`
   et `crates/kesh-i18n/locales/*/messages.ftl` ; chaque site est juste (achat, solde) ou corrigé (règlement). Un grep vide sur un symptôme corrigé éveille le soupçon.
3. **Le manuel, PDF compris** : recompile dans une **copie** de `docs/manual` placée sous `/tmp/claude-1000/` (jamais dans le worktree), `make fr/user-manual.pdf
   fr/admin-manual.pdf`, aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), greppe les trois jetons ci-dessus.
4. Aucun fichier de production touché par le commit : `git diff --stat <SHA>^..<SHA>` ne nomme que `docs/manual/fr/*.tex` (et la fiche) ; sinon, dis-le et rends la main.
5. Gate : `docs/manual` se compile sans référence indéfinie ; aucun test Rust ni Vitest n'est requis pour un commit de manuel seul.

## Interdits

Tu n'écris **aucun fichier** du dépôt (`Edit`, `Write`, `git commit`, `git push`, `sed -i`, `scripts/prepare-release.sh`, tout script qui écrit, `gh` en écriture,
`docker`, `GRANT`, `REVOKE`, `pkill`/`killall`/`kill` par motif). Rien sous `~/kesh/`. Ton rapport va dans `$GATE_LOGS/e63/63-1-revue-p3.md`, seul fichier écrit.
**Guardrail Haiku** : tout finding CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence d'un anti-pattern se vérifie par `grep -nF "<chaîne>" <fichier>` avant d'être
rendu ; un finding réfuté est écarté et consigné.

## Rendu

Chaque finding : CRITICAL, HIGH, MEDIUM ou LOW, avec fichier:ligne, la prémisse lue au code et le correctif proposé ; puis la liste des axes **exercés et non exercés**.
Un « 0 finding » sans cette liste ne compte pas.
