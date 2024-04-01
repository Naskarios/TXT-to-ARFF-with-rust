#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unreachable_code)]
use std::borrow::Borrow;
use std::fmt::Error;
use std::fs::{read, File};
use std::io::{BufRead, BufReader};
use std::io::{Seek, Write};
use std::process::exit;
use std::{error, io, result, vec};

fn main() -> io::Result<()> {
    println!("HELLO!!!!");
    //filepaths,change accordingly
    let agePath = "./../customerAge.txt";
    let ageWekaPath = "./../empAge.arff";
    let buysPath = "./../customerBuys.txt";
    let buysWekaPath = "./../empProduct.arff";
    let catPath = "./../categories.txt";
    let catWekaPath = "./../empCAT.arff";

    //ABOUT CATEGORIES

    productClustering(buysPath, catPath, catWekaPath);
    println!("DONE CATEGORIES");

    //**********************************
    // ABOUT AGES

    let mut textFile = File::open(agePath)?;
    let mut wekaFile = File::create(ageWekaPath)?;
    let mut reader = io::BufReader::new(textFile);

    let headerText = "@relation \"empAge\"\n@attribute id string\n@attribute age numeric\n";

    writeln!(wekaFile, "{}", headerText); //writing the arff header
    writeln!(wekaFile, "@data");

    println!("WRITING AGES");
    formatWriteARFF(reader, wekaFile);
    println!("DONE AGES");

    //**********************************
    // ABOUT PRODUCTS

    //re init our file pointers and reader
    textFile = File::open(buysPath)?;
    wekaFile = File::create(buysWekaPath)?;
    reader = io::BufReader::new(textFile);

    let productNominalStack = &nominalStack(reader, 2);

    textFile = File::open(buysPath)?;
    reader = io::BufReader::new(textFile);

    let dateNominalStack = &nominalStack(reader, 1);

    textFile = File::open(buysPath)?;
    reader = io::BufReader::new(textFile);

    writeln!(
        wekaFile,
        "{}",
        "@relation \"empProduct\"\n@attribute purchaseDate ".to_owned()
            + &dateNominalStack
            + "\n@attribute id "
            + &nominalStack(reader, 0)
            + "\n@attribute product "
            + productNominalStack //to view in weka the count for every product
    ); // FIRST PART OF THE HEADER

    //re init because it was "moved during" nominalstack(),just rust things
    textFile = File::open(buysPath)?;
    reader = io::BufReader::new(textFile);

    //en telei den xreiazetai giauto einai se comment h for loop stin transactionformatWriteARF
    let dictionaryHeaderIDTuple = createDictionaryWriteHeader(reader, 0);

    textFile = File::open(buysPath)?;
    reader = io::BufReader::new(textFile);

    let dictionaryHeaderProductTuple = createDictionaryWriteHeader(reader, 2);

    writeln!(wekaFile, "{}", dictionaryHeaderProductTuple.1);

    textFile = File::open(buysPath)?;
    reader = io::BufReader::new(textFile);

    writeln!(wekaFile, "@data");
    transactionformatWriteARFF(
        reader,
        wekaFile,
        dictionaryHeaderProductTuple,
        dictionaryHeaderIDTuple,
    );

    println!("DONE PRODUCTS");

    println!("BYEEEE!!!!!!");
    Ok(())
}

fn transactionformatWriteARFF(
    // writing the @data section,not the @data tho
    reader: BufReader<File>,
    mut wekaFile: File,
    tupleProduct: (Vec<String>, String),
    tupleID: (Vec<String>, String),
) {
    for line in reader.lines() {
        match line {
            Ok(content) => {
                // transforming the selected text to its .arff counterpart
                //one line at a time
                let content2 = content.replace('\t', ","); //the tabs
                let lineParts: Vec<&str> = content2.split(',').collect(); //collecting all of them in one vector
                let mut data: String = String::new();
                let greeklish = transliterate_greek_to_english(lineParts[2]); //greek characters arent viewed properly by weka

                data.push_str(lineParts[1]);
                data.push(',');
                data.push_str(lineParts[0]);
                data.push(',');
                data.push('\"');
                data.push_str(&greeklish);
                data.push('\"');
                data.push(',');
                // for v in &tupleID.0 {
                //     match v.starts_with(lineParts[0]) {
                //         true => data.push('t'),
                //         false => {
                //             data.push('?');
                //         }
                //     }

                //     data.push(',');
                // }
                for v in &tupleProduct.0 {
                    match v.starts_with(&greeklish) {
                        true => data.push('t'),
                        false => {
                            data.push('?');
                        }
                    }
                    data.push(',');
                }
                data.pop();
                writeln!(wekaFile, "{}", data); // the "{}" acts as a placeholder which is replaced by line.unwrap
            }
            Err(error) => {
                println!("something's wrong here reader.lines {}", error);
            }
        }
    }
}

fn createDictionaryWriteHeader(reader: BufReader<File>, col: usize) -> (Vec<String>, String) {
    //creates the @attribute part of the header for a nominal-transaction type  and
    //returns a vector with every possible item detected
    let mut headerString = String::new();
    let mut dictionaryVec: Vec<String> = Vec::new(); //the explaination if the vector's existance is in the if statement below
    for line in reader.lines() {
        match line {
            Ok(content) => {
                let content2 = content.replace('\t', ",");
                let lineParts: Vec<&str> = content2.split(',').collect(); //collecting all of them in one vector
                let greeklish = transliterate_greek_to_english(lineParts[col]);
                if !dictionaryVec.contains(&greeklish) {
                    // AMA KOITAW  STRING pou PX. exei to "boyturomeno" ama psaksw na dw ama yparxei to "boutyro" den tha mpei stin lista
                    //kai auto einai problima giati yparxoun times mh dhlwmenes ston header me apotelesma na min to dexete to WEKA
                    //etsi katalhfw ston vector
                    dictionaryVec.push(greeklish);
                }
            }
            Err(error) => {
                println!("ERROR {}", error);
            }
        }
    }
    headerString.push('\n');
    for v in &dictionaryVec {
        let temp: String = "@attribute \'".to_owned() + v + "\' { t}\n";
        headerString.push_str(&temp);
    }

    (dictionaryVec, headerString)
}

fn nominalStack(reader: BufReader<File>, col: usize) -> String {
    //creates a nominal attr but not the "transaction type" for apriori
    let mut headerString = String::new();
    let mut dictionaryVec: Vec<String> = Vec::new(); //the explaination if the vector's existance is in the if statement below
    for line in reader.lines() {
        match line {
            Ok(content) => {
                let content2 = content.replace('\t', ",");
                let lineParts: Vec<&str> = content2.split(',').collect(); //collecting all of them in one vector
                let greeklish = transliterate_greek_to_english(lineParts[col]);
                if !dictionaryVec.contains(&greeklish) {
                    // AMA KOITAW  STRING pou PX. exei to "boyturomeno" ama psaksw na dw ama yparxei to "boutyro" den tha mpei stin lista
                    //kai auto einai problima giati yparxoun times mh dhlwmenes ston header me apotelesma na min to dexete to WEKA
                    //etsi katalhgw ston vector
                    dictionaryVec.push(greeklish);
                }
            }
            Err(error) => {
                println!("ERROR {}", error);
            }
        }
    }
    headerString.push('{');
    for v in dictionaryVec {
        let temp: String = "\'".to_owned() + &v + "\',";
        headerString.push_str(&temp);
    }
    headerString.pop();
    headerString.push('}');

    headerString
}

fn transliterate_greek_to_english(input: &str) -> String {
    let mut result = String::new();

    for c in input.chars() {
        match c {
            'α' | 'Α' | 'ά' => result.push('a'),
            'β' | 'Β' => result.push('b'),
            'γ' | 'Γ' => result.push('g'),
            'δ' | 'Δ' => result.push('d'),
            'ε' | 'Ε' | 'έ' => result.push('e'),
            'ζ' | 'Ζ' => result.push('z'),
            'η' | 'Η' | 'ή' => result.push('h'),
            'θ' | 'Θ' => result.push_str("th"),
            'ι' | 'Ι' | 'ί' | 'ϊ' => result.push('i'),
            'κ' | 'Κ' => result.push('k'),
            'λ' | 'Λ' => result.push('l'),
            'μ' | 'Μ' => result.push('m'),
            'ν' | 'Ν' => result.push('n'),
            'ξ' | 'Ξ' => result.push('x'),
            'ο' | 'Ο' | 'ό' => result.push('o'),
            'π' | 'Π' => result.push('p'),
            'ρ' | 'Ρ' => result.push('r'),
            'σ' | 'Σ' => result.push('s'),
            'τ' | 'Τ' => result.push('t'),
            'υ' | 'Υ' | 'ύ' => result.push('y'),
            'φ' | 'Φ' => result.push('f'),
            'χ' | 'Χ' => result.push('x'),
            'ψ' | 'Ψ' => result.push_str("ps"),
            'ω' | 'Ω' | 'ώ' => result.push('o'),
            'ς' => result.push('s'),
            _ => result.push(c),
        }
    }

    result
}

fn formatWriteARFF(reader: BufReader<File>, mut wekaFile: File) {
    println!("Entering line loop");
    for line in reader.lines() {
        match line {
            Ok(content) => {
                //arr[5] to \t
                // println!("{:?}", content);
                let content2 = content.replace('\t', ",");
                writeln!(wekaFile, "{}", content2); // the "{}" acts as a placeholder which is replaced by line.unwrap
            }
            Err(error) => {
                println!("something's wrong here reader.lines {}", error);
            }
        }
    }
}

fn productClustering(
    buysPath: &str,
    catPath: &str,
    catWekaPath: &str,
) -> Result<(), std::io::Error>
//categories according to the vectors,
//doesnt ouput anything just writes the weka file directly
{
    // ******CATEGORIES
    let delicatesen: Vec<&str> = vec![
        "delicatesen",
        "λαχανικά ντελικατέσεν",
        "ποτά ντελικατέσεν",
        "λίπος ντελικατέσεν",
        "σοκολάτα ντελικατέσεν",
        "τυρί ντελικατέσεν",
    ];
    let bread_vector: Vec<&str> = vec![
        "bread",
        "άσπρο ψωμί",
        "μαύρο ψωμί",
        "προϊόν αρτοποιίας μακράς διαρκείας",
        "ημιψημένο ψωμί",
        "ρολά/ψωμάκια",
        "ζύμη",
        "παξιμάδι",
    ];
    let cleaning_vector: Vec<&str> = vec![
        "cleaning",
        "απορρυπαντικό",
        "καθαριστικό τουαλέτας",
        "καθαριστικό πιάτων",
        "διαβρωτικό/καθαριστικό",
        "καθαριστικό μπάνιου",
        "καθαριστικό",
    ];
    let pet_vector: Vec<&str> = vec!["pet", "φροντίδα κατοικίδιου", "γατοτροφή", "σκυλοτροφή"];
    let frozenfood_vector: Vec<&str> = vec![
        "frozenfood",
        "έτοιμες σούπες",
        "κατεψυγμένα προϊόντα πατάτας",
        "κατεψυγμένα γεύματα",
        "έτοιμα προϊόντα διατροφής",
        "κατεψυγμένο κοτόπουλο",
        "κατεψυγμένα ψάρια",
        "κατεψυγμένα φρούτα",
        "σούπες",
    ];
    let hygiene_beauty_vector: Vec<&str> = vec![
        "hygiene",
        "προιόντα υγιεινής",
        "οδοντιατρική φροντίδα",
        "φροντίδα δέρματος",
        "σπρέι μαλλιών",
        "γυναικεία είδη υγιεινής",
        "αφαίρεση μακιγιάζ",
        "ανδρικά καλλυντικά",
        "καλλυντικά για μωρά",
    ];
    let vegetable_vector: Vec<&str> = vec![
        "vegetable",
        "προϊόντα πατάτας",
        "άλλα λαχανικά",
        "λαχανικά τουρσί",
        "κονσερβοποιημένα λαχανικά",
        "κρεμμύδια",
        "ριζώδη λαχανικά",
        "κατεψυγμένα λαχανικά",
    ];
    let drink_vector: Vec<&str> = vec![
        "drink",
        "ποτά",
        "ουίσκι",
        "λικέρ (ορεκτικό)",
        "λικέρ",
        "αφρώδες κρασί",
        "κρασί prosecco",
        "ρούμι",
        "κονιάκ",
        "λευκό κρασί",
        "διάφορα ποτά",
        "κόκκινο/ροζέ κρασί",
        "κονσερβοποιημένη μπύρα",
    ];
    let meat_vector: Vec<&str> = vec![
        "meat",
        "κρέας",
        "βιολογικό λουκάνικο",
        "βοδινό κρέας",
        "κοτόπουλο",
        "κρέας για άλειμμα",
        "λουκάνικο",
        "κρέας για μπέργκερ",
        "χοιρινό",
        "ζαμπόν",
        "γαλοπούλα",
        "λουκάνικο φρανκφούρτης",
        "συκώτι",
    ];
    let fish_vector: Vec<&str> = vec!["fish", "ψάρι", "κονσερβοποιημένο ψάρι"];
    let gardening_vector: Vec<&str> = vec![
        "garden",
        "λουλούδια (σπόροι)",
        "γλάστρες",
        "λίπασμα για λουλούδια",
        "βότανα",
    ];
    let sauce_vector: Vec<&str> = vec!["sauce", "σάλτσα σαλάτας", "σάλτσες"];
    let beverage_vector: Vec<&str> = vec![
        "beverage",
        "εμφιαλωμένο νερό",
        "χυμός φρούτων/λαχανικών",
        "καφές",
        "τσάι",
        "ποτά κακάου",
        "εμφιαλωμένη μπύρα",
        "στιγμιαίος καφές",
    ];

    let bag_vector: Vec<&str> = vec![
        "bags",
        "μεμβράνη/σακούλες φαγητού",
        "τσάντες αγορών",
        "τσάντες",
    ];
    let bath_vector: Vec<&str> = vec!["bath", "σαπούνι", "οινόπνευμα", "μαλακτικό"];
    let kitchen_vector: Vec<&str> = vec![
        "kitchen",
        "κουζινικά",
        "αφαλατωτής",
        "μαγειρικά σκεύη",
        "προϊόντα οικιακής χρήσης",
        "πετσέτες κουζίνας",
        "πιάτα",
        "χαρτοπετσέτες",
    ];
    let food_vector: Vec<&str> = vec![
        "trofima",
        "μουστάρδα",
        "ντόπια αυγά",
        "μπάρα δημητριακών",
        "δημητριακά",
        "μαρμελάδα",
        "κέτσαπ",
        "μούσταρδα",
        "μαγιονέζα",
        "ρύζι",
        "μπέικιν πάουντερ",
        "καραμέλα",
        "ζυμαρικά",
        "πουτίγκα σε σκόνη",
        "σαντιγί",
    ];
    let prepeiNaExeiToSpiti: Vec<&str> = vec![
        "mustHave",
        "μέλι",
        "μπαχαρικά",
        "ξύδι",
        "ζάχαρη",
        "σόδα",
        "υγρό",
        "αλάτι",
        "λάδι",
        "αλεύρι",
    ];
    let snack_vector: Vec<&str> = vec![
        "snack",
        "μεζέδες",
        "αλμυρό σνακ",
        "σνακ",
        "ποπ κορν",
        "ξηροί καρποί",
        "προϊόντα σνακ",
        "τσίχλα",
    ];
    let dessert_vector: Vec<&str> = vec![
        "dessert",
        "επιδόρπιο",
        "κρέμα",
        "σιρόπι",
        "βάφλες",
        "σοκολάτα κουβερτούρα",
        "παγωμένο επιδόρπιο",
        "σοκολάτα υγείας",
        "σοκολάτα",
        "παγωτό",
        "γλυκά για άλειμμα",
    ];
    let dairy_vector: Vec<&str> = vec![
        "dairy",
        "γάλα μη αποβουτυρωμένο",
        "συμπυκνωμένο γάλα",
        "ξινόγαλο",
        "βούτυρο",
        "γιαούρτι",
        "γάλα μακράς διάρκειας",
        "τυρόγαλο",
        "μαργαρίνη",
    ];
    let cheese_vector: Vec<&str> = vec![
        "cheese",
        "μαλακό τυρί",
        "τυρί για άλειμμα",
        "σκληρό τυρί",
        "μεταποιημένο τυρί",
        "τυρί κρέμα",
        "φέτες τυριού",
    ];
    let fruit_vector: Vec<&str> = vec![
        "fruit",
        "φρούτα με κουκούτσι",
        "φρούτα σε κονσέρβα",
        "συσκευασμένα φρούτα/λαχανικά",
        "σταφύλια",
        "τροπικά φρούτα",
        "εσπεριδοειδή",
        "μούρα",
    ];
    let clothes_vector: Vec<&str> = vec!["clothes", "ρούχα"];

    let photography_vector: Vec<&str> = vec!["movie", "φωτογραφία/ταινία"];

    let newspapers_vector: Vec<&str> = vec!["newspaper", "εφημερίδες"];

    let candles_vector: Vec<&str> = vec!["candles", "κεριά"];
    let lights_vector: Vec<&str> = vec!["lampes", "λάμπες"];

    let artificial_sweeteners_vector: Vec<&str> = vec!["sweetener", "τεχνητή γλυκαντική ουσία"];

    let maintenance_products_vector: Vec<&str> =
        vec!["maintenance Products", "προϊόντα συντήρησης"];
    let combined_vector: Vec<&str> = vec![
        "products?",
        "εποχιακά προϊόντα",
        "βιολογικά προϊόντα",
        "προϊόντα σε ρολό",
    ];

    let listOfcategories = [
        &delicatesen,
        &prepeiNaExeiToSpiti,
        &meat_vector,
        &lights_vector,
        &cheese_vector,
        &frozenfood_vector,
        &drink_vector,
        &vegetable_vector,
        &hygiene_beauty_vector,
        &pet_vector,
        &cleaning_vector,
        &bread_vector,
        &fish_vector,
        &gardening_vector,
        &sauce_vector,
        &beverage_vector,
        &bag_vector,
        &bath_vector,
        &kitchen_vector,
        &food_vector,
        &snack_vector,
        &dessert_vector,
        &dairy_vector,
        &fruit_vector,
        &clothes_vector,
        &photography_vector,
        &newspapers_vector,
        &candles_vector,
        &artificial_sweeteners_vector,
        &maintenance_products_vector,
        &combined_vector,
    ];

    let testFile = File::open(buysPath)?;
    let mut catFile = File::create(catPath)?;
    let reader = io::BufReader::new(testFile);

    for line in reader.lines() {
        // creating the categories by checking
        //if the product in the lineparts exists in our collection of vectors
        match line {
            Ok(content) => {
                let lineParts: Vec<&str> = content.split('\t').collect(); //collecting all of them in one vector
                for cat in listOfcategories {
                    for thing in cat {
                        if thing.eq_ignore_ascii_case(lineParts[2]) {
                            let data =
                                lineParts[0].to_owned() + "\t" + lineParts[1] + "\t" + cat[0];
                            writeln!(catFile, "{}", data); //"rewriting" the customersBuys but now with categories
                        }
                    }
                }
            }
            Err(error) => {
                println!("something's wrong here reader.lines {}", error);
            }
        }
    }

    //re init our file pointers and reader
    let mut textFile = File::open(catPath)?;
    let mut wekaFile = File::create(catWekaPath)?;
    let mut reader = io::BufReader::new(textFile);

    writeln!(
        wekaFile,
        "{}",
        "@relation \"empProduct\"\n@attribute purchaseDate date dd/mm/yy\n@attribute id "
            .to_owned()
            + &nominalStack(reader, 0)
    ); // FIRST PART OF THE HEADER

    textFile = File::open(catPath)?;
    reader = io::BufReader::new(textFile);
    writeln!(
        wekaFile,
        "{}",
        "@attribute prodcut ".to_owned() + &nominalStack(reader, 2)
    );

    textFile = File::open(catPath)?;
    reader = io::BufReader::new(textFile);

    let dictionaryHeaderIDTuple = createDictionaryWriteHeader(reader, 0);

    textFile = File::open(catPath)?;
    reader = io::BufReader::new(textFile);

    let dictionaryHeaderProductTuple = createDictionaryWriteHeader(reader, 2);

    textFile = File::open(catPath)?;
    reader = io::BufReader::new(textFile);

    //write the headers
    // writeln!(wekaFile, "{}", dictionaryHeaderIDTuple.1); // the "{}" acts as a placeholder which is replaced by line.unwrap
    writeln!(wekaFile, "{}", dictionaryHeaderProductTuple.1);

    writeln!(wekaFile, "@data");
    transactionformatWriteARFF(
        reader,
        wekaFile,
        dictionaryHeaderProductTuple,
        dictionaryHeaderIDTuple,
    );
    Ok(())
}
