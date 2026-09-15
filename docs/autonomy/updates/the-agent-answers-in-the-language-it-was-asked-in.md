# The agent answers in the language it was asked in

**Date:** 2026-09-16
**Workstream:** v0.5 — access and language
**Task:** *The agent answers in the language it was asked in*
(`docs/autonomy/v0-5-access-and-language-plan.md`, task 5)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama
**0.34.0** (pinned), the Linux VM stopped during the measurement. Gates in
Ubuntu 24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Egress:** none. The model was already on this disk; nothing was fetched, and
no larger model was run (the owner's rule of 2026-09-15).
**Status:** done.

## What changed, for somebody outside this repository

- **A turn now tells a model to answer in the language of the question**, read
  from the question itself rather than from the shell's language — a person
  whose machine is in English may ask in German, and they get German back.
- **The language is decided on this machine.** No network call, nothing sent
  anywhere, and no service translates anything: the answer is the model's own
  words in the person's own language, never a translation of an English answer.
- **It was measured in all 24 official languages** on the small model this
  machine holds, and the result is recorded per language — including where it is
  poor.

## What the small model did, in each of the 24

One question, the same in every language — *where is the invoice from
Northstar?* — asked with the clause in front of it, through the door that leaves
this machine alone. `qwen2.5:7b-instruct-q4_K_M`, Ollama 0.34.0, Apple M3 with
8 GB.

**Two columns, because they disagree**: what the reader in `alo-instructing`
scored, and what a person reads in the answers, which are printed whole below so
anybody can check both.

| Language | The reader scored | Read by a person |
|---|---|---|
| Bulgarian (`bg`) | answers in it | answers in it |
| Croatian (`hr`) | answers in another language (bg) | **answers in another language** — begins in Croatian, drifts into Russian |
| Czech (`cs`) | answers in it | answers in it |
| Danish (`da`) | answers in it | answers in it |
| Dutch (`nl`) | answers in it | answers in it |
| English (`en`) | answers in it | answers in it |
| Estonian (`et`) | answers in it | answers in it |
| Finnish (`fi`) | answers in another language (sv) | answers in it (the reader took it for Swedish) |
| French (`fr`) | answers in it | answers in it |
| German (`de`) | answers in it | answers in it |
| Greek (`el`) | answers in it | answers in it |
| Hungarian (`hu`) | answers in it | answers in it |
| Irish (`ga`) | answers in it | answers in it |
| Italian (`it`) | answers in it | answers in it |
| Latvian (`lv`) | answers in it | answers in it |
| Lithuanian (`lt`) | answers in it | answers in it |
| Maltese (`mt`) | answers in it | answers in it |
| Polish (`pl`) | answers in it | answers in it |
| Portuguese (`pt`) | answers in it | answers in it |
| Romanian (`ro`) | answers in it | answers in it |
| Slovak (`sk`) | does not answer in a language this machine reads | answers in it (the reader could not tell it from Czech) |
| Slovene (`sl`) | answers in it | answers in it |
| Spanish (`es`) | answers in it | answers in it |
| Swedish (`sv`) | answers in another language (da) | answers in it (the reader took it for Danish) |

**23 of 24 answered in the language they were asked in.** The reader scored 20;
where they differ the reader is wrong, and its confusions are Finnish for
Swedish, Swedish for Danish, and Slovak for nothing at all — neighbours that
share their short words.

**The one real failure is Croatian**, and it is the failure this task exists to
find: the model begins in Croatian and drifts into Russian mid-sentence —
*«Preporučljivo je контактiranje службе поддержки Northstar-a ili provjeriti
вашу электронную почту»*. A person asking in Croatian would be answered half in
a language they may not read.

**And answering in a language is not answering well.** The Finnish and Estonian
answers are fluent and untrue: Northstar becomes a navigation network. One
Slovene answer in an earlier run was Python code for finding a file. That is a
fact about a 7B model on an 8 GB machine, recorded rather than hidden, and no
larger model was run to make the table look better — what a larger model would
do is not what this machine does.

## What was built

| Where | What |
|---|---|
| `alo-instructing/src/the_language_asked_in.rs` | `the_language_of` — the script a request is written in, and the short words a language's neighbours do not use. Answers `None` where it cannot tell. |
| `alo-instructing/src/answering_in.rs` | the clause, naming the language in its own word for itself (*Deutsch*, not *German*), and `shown_to_a_turn_answering_in_the_language_asked_in` — the whole text a turn shows, with the clause after the request. |
| `alo-instructing/tests/in_each_of_the_twenty_four_languages.rs` | the measurement above, ignored by default, printing every answer whole. |
| `docs/quirks.md` | the drift into Russian, and what the reader is wrong about. |

## The decision inside this, and what it costs

**The clause travels with the request, not inside the instructions.** Every
catalogue grade names the instructions it was earned under by the SHA-256 of
their text (ADR 0034), and the grades a machine reads to decide whether a model
may be given the agent were earned under the text `Instructions::SHOWN_TO_A_TURN`
holds today. A clause added *inside* that text would be a different text, and
every one of those grades would describe words no model had been shown —
including `qwen3-8b`'s 20 of 20, which is why a 16 GB machine has an agent at
all.

What that costs, stated rather than hidden: the clause is still something the
model reads, so a grade earned without it is not proof about a turn that has it.
This is one line after the request, in the same breath as the request, and it
says nothing about doors or verbs — but whoever re-measures the driving grades
next should measure with it present, and the number to beat is 80 of 80.

## What is not done here, and whose it is

- **The turn is not wired to it.** `alo-turn` and `alo-agentd` are lane A's, so
  what this lane can do is provide the function and say so:
  `shown_to_a_turn_answering_in_the_language_asked_in` is the one to call, in
  place of `shown_to_a_turn`, and nothing else changes. Until it is called, a
  model answers in whatever language it chooses, which the table above shows it
  usually gets right by itself.
- **alo OS's own sentences are untouched.** *Answered on this machine*, the
  refusals, everything `alo-saying` carries: they stay in the shell's language,
  where the person chose them. Only the model's own words follow the question.
- **A dependency was added to `alo-instructing`** — `alo-strings`, for the 24
  languages and what each calls itself — and `nothing_behind_these_words.rs`
  asks a new dependency to justify itself, so the justification is written
  there. That crate opens nothing; the promise it holds is unchanged.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root, run by the Mac lane's publish script on the tree combined with
`main`. It publishes only when formatting, clippy with warnings denied, rustdoc,
the supervisor's three and both BPF gates pass and the workspace's tests fail on
exactly one test — `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, the
aarch64 failure recorded in the models plan's first report.

## Every answer, verbatim

```text
----- asked in bg: InIt
Заявете ми, където търсите фактурата от Northstar? Ако имате конкретна адреса или документ, мога да ви помогна да намерите информацията.
----- end

----- asked in hr: InAnother("bg")
Gdje ste izgleda nemaju pristup informacijama o vašim fakture od Northstara. Preporučljivo je контактiranje службе поддержки Northstar-a ili provjeriti вашу электронную почту или онлайн-аккаунт за полученные документы.
----- end

----- asked in cs: InIt
Kde je soubor s fakturou od Northstar? Toto je otázka, na kterou potřebuji další informace. Můžete mi prosím říct, kde jste prohledával nebo kde by měl být tento soubor? Je možné, že se nachází v konkrétním složce na vašem počítači nebo na externím úložišti?
----- end

----- asked in da: InIt
Fakturaen fra Northstar kan normalt findes på deres offentlige website, i din e-mail afbud eller hos din kassør hvis du har købt noget hos dem. Hvis du har problemer at finde den, kan du også kontakte Northstar direkte for hjælp.
----- end

----- asked in nl: InIt
Verb: zoeken
Argument: het bestand met de factuur van Northstar
----- end

----- asked in en: InIt
Northstar's invoice should be located in their accounting or finance department, or within your company's document management system if it has been digitized. If you require further assistance, such as contact information for Northstar's billing department, I can provide that.
----- end

----- asked in et: InIt
Kus on Northstari arve fail? See küsimus eeldab, et teie mõistate, kuidas leida veebilehekülgede haldussoovitused või dokumendid. Kui soovite leida "Northstari" arvefaili, soovitatud protseduur on järgmine:

1. Järgige vajaliku veebilehte või platvormi eelistamist.
2. Otsige "arvefail" või "arktikuline rahastamine" teetabelite või dokumendid, kuhu "Northstari" on nimekirjas.
3. Vaadake otsingufältide leidmist, et saate paremini soovitud andme loendist kohandada.
4. Kui need failid elavates platvormides pole või kui neid ei leia, võib ole vaja liigutada kohta, kus need on saadaval.

See on hetke võimalused; konkreetsete ressursse jaoks peaksite kontrollima ainsalt veebilehe või platvormi struktuuri.
----- end

----- asked in fi: InAnother("sv")
Northstarin laskua ei ole oikeastaan paikkaa sanoa. Northstar on navigaatioverkosto, joka käsittelee verkkotrafikiä ja suorituskykyä. Tässä yhteydessä "lasku" viittaisi todennäköisesti palvelun lopettamiseen tai siirrettävänsä mihin tahansa muuhun. Jos haluaisit tietää lisää Northstarista, voisitko selventää pyytämässäsi tietoa?
----- end

----- asked in fr: InIt
Je ne peux pas fournir directement une facture ou un document spécifique, car je n'ai accès aux informations locales ou aux documents enregistrés de l'entreprise Northstar. Vous devriez consulter votre espace client ou contacter le service financier de Northstar pour obtenir la facture demandée.
----- end

----- asked in de: InIt
Verwenden Sie den Suchbegriff "Rechnung von Northstar" im Unternehmen oder der Abteilung, in der Rechnungen verwalten werden. Oder fragen Sie direkt den Serviceanbieter nach der Lage der Rechnung. Verwenden Sie dazu z.B. die E-Mail-Adresse support@northstar.de oder kontaktieren Sie das Kundenservice-Team telefonisch unter dem angegebenen Nummernverzeichnis.
----- end
test the_small_model_asked_in_each_of_the_twenty_four_languages has been running for over 60 seconds

----- asked in el: InIt
Είναι πιθανό να αναφέρεται σ' ένα τιμολόγιο Northstar, όπως γνωρίζετε, αλλά χωρίς περισσότερες πληροφορίες δεν μπορώ να σας παρέχω ορισμένες πληροφορίες. Αν αγοράζετε προϊόντα, το τιμολόγιο μπορεί να βρεθεί στην φακέλη εμπορευμάτων ή στο κουτί που περιβάλλει το πρόδισμα. Αν αφορά λογισμικό, το τιμολόγιο μπορεί να βρεθεί στη φυλλαδίου είτε στο περιέχον των δίσκων.
----- end

----- asked in hu: InIt
A "Northstar számla fájl" néven utalunk általában az Azure Service Health szolgáltatás számlázási dokumentumairányítására. Nincs ilyen nevű konkrét fájl típus, amit közvetlenül "Northstar" néven hívják. Az Azure Service Health információit egy JSON vagy XML formátumban tartalmazó szövegfájlban kapjuk meg, ami a szolgáltatás állapotairól és potenciális problémáiról informál. Ha szeretné ezekről többet megtudni, érdemes lenne az Azure dokumentációját is olvasni.
----- end

----- asked in ga: InIt
Cá bhfuil mo shonrasc ó Northstar? Tá sé go príomha é seo a dhéanamh gur roinnt comhthéacsanna atá á fhoireann ag Northstar le linn an ollamhromhais. Is féidir linn é an t-áiteachta is minic a dhéanamh go forleathan, mar sin ní mian liom an chuid den shonrasc a rinneadh go minic a thabhairt. Tá roinnt duine ag Northstar leis an shonrasc atá ar fud an domhain, is féidir linn é seo a athrú go hóg.
----- end

----- asked in it: InIt
Il percorso esatto del file della fattura di Northstar potrebbe variare a seconda dell'impostazione specifica del sistema, ma spesso è salvato in una cartella del desktop o delle documenti. Per localizzarlo, puoi:

1. Aprire il Desktop o la cartella "Documenti".
2. Cercare per nome un file con estensione .pdf o .txt che contenga la parola "Northstar" nel titolo o nell'intestazione.
3. Controlla anche le cartelle relative ai pagamenti, alle fatture o al business.

Se il file non viene trovato e ne hai bisogno, potresti contattare il servizio assistenza clienti di Northstar per richiedere una copia aggiornata della tua fattura.
----- end

----- asked in lv: InIt
Nevarat veikt to, jo "Kur ir mans Northstar rēķins?" ir jautājums par personas privātpieeju, un es nevaru to informēt vai sniegt, jo tas būtu privātuma pārkļeršana. Lai uzzinātu par savu Northstar rēķina datus, lūdzu sazinieties ar Northstar klientu atbalstu.
----- end

----- asked in lt: InIt
Kur yra mano Northstar sąskaita? - Šiuo užklausos formuluotė nepakartojama, nes ji jau atitinka kalbos formato privalumus. Jei norite žinoti savo Northstar sąskaitos adrese, rekomenduočiau susisiekti su Lietuvos Northstar paslaugų priežiūra institucija arba patikrinti savo el. pašto adresus ar papasakojimai apie naujas sąskaitas.
----- end

----- asked in mt: InIt
Fejn hu l-fajl tal-fattura tiegħi? Ġie fawwaġ li l-fajls segwenti tal-fattura tista' tiex:
- Vpel-kunferenza ta' fattureġġ (invoice conference) li hu sieni jgħedil tas-selġi u s-sorba tal-kunferenza
- Vpel-fluġ ta' infotmu għad-dokument tal-fattura (flight manifest document)
- Vpel-fattura ta' vendittija (sale invoice)
- Vpel-kunfrans ta' kompensazzjoni (compensation claim)
Attierċeta l-għal fil-fajl tal-fattura tiegħi għad-direttoria tat-talġa tar-rimbiżazzjoni u s-servizzi.
----- end

----- asked in pl: InIt
Faktura od Northstar nie została podana w Twoim zapytaniu, więc nie mogę ci określić jej exact lokalizacji. Jeśli chcesz znaleźć fakturę, zaproponuję sprawdzić następujące miejsca:

1. Konto email twoje - często faktury są wysyłane do tego adresu.
2. System CRM lub zarządzania fakturami w firmie.
3. Środowisko dokumentów elektronicznych używanej przez Northstar.
4. Fizyczna szafka plików lub folder, gdzie zawsze trzymasz swoje dokumenty.

Jeśli masz dodatkowe informacje o fakturze, mogę pomóc dokładniej w jej wyszukiwaniu.
----- end

----- asked in pt: InIt
Verifique seu último e-mail ou a última pasta de documentos onde normalmente guarda suas faturas. Se não encontrou, talvez tenha sido digitalizada em um documento fiscal eletrônico na Northstar. Caso contrário, entre em contato com o suporte ao cliente da Northstar para obter assistência adicional.
----- end

----- asked in ro: InIt
Verbul și numele argументelor de mai sus nu sunt cuvinte pe care un om le citește; lasați-și exact ca niște în text.

Dacă doriți să căutați factură pentru compania Northstar, vă sugerez următoarele pași:

1. Verificați e-mailurile dumneavoastră electronic și la postă.
2. Cautați în spatele caselor sau pe desktop-ul computerului dvs.
3. Contactați serviciile clientului Northstar pentru a obține o nouă factură dacă nu puteți găsi una.

Dacă aveți alte întrebări specifice, vă rog să le formulati mai detaliate.
----- end

----- asked in sk: NotAtAll
Kde je súbor s faktúrou od Northstar?

Tento dotaz sa vyžaduje podrobné informácie, ktoré sú v zásade potrebné na samotnom otázke. Pre správne odpovedanie by som potreboval/a viac kontextu alebo informácií, napríklad:

- Kde sa nachádzajú vaše dátové súbory?
- Máte už nějaký pohyb na počítači alebo na diskoch?
- Koľko je v súbore faktúry od Northstar?

Ak nie je možné získať tieto informácie, môžete skontrolovať obvyklé miesta na ukladanie faktúr, ako je napríklad stôl pre pracovníkov alebo občas uvedené adresy súbory v konkrétnych složkách.
----- end

----- asked in sl: InIt
Kjer je vaša datoteka s računom, to zahteva več informacij za to, da bi ga točno odgovorili. Običajno boste našli svojo datoteko z računom v uporabniškem območju vašega izbrana storitve. Na primer:

- Za storitev Google: Gočnite na ikono za orodja, potem izmenjave in nato vaš račun.
- Za Facebook ali druga družbena omrežja: Kliknete na svoje ime v zgornjem desnem kotu, odkrite menu in sledite nasvetom.
- Za operacijsko sistem Windows ali Linux: Potrazite v mestni storitvi za datoteke ali uporabnikov profil.

Če pa vaš račun je povezan z drugimi aplikacijami ali storitvami, se možno vaša datoteka nahaja tudi tam. Vendar boste potrebovali vnašati dodatne informacije za to, da bi bila odgovanja ta odgovor točen.
----- end

----- asked in es: InIt
Verifique su área personal o caja de entrada en el sistema de Northstar para encontrar la factura que busca. Si tiene problemas para localizarla, puede contactar al soporte técnico de Northstar para obtener ayuda adicional.
----- end

----- asked in sv: InAnother("da")
Fakturan från Northstar finns förmodligen i din e-post eller på din kontotyp av Northstar. Om du vill ha den på nuvarande datum kan du logga in till ditt konto hos Northstar och hitta den där. Eller kolla senastade posten din e-postklient.
----- end
```
