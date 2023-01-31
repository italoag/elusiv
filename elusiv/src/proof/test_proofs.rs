use super::proof_from_str_projective;
use crate::{
    fields::u256_from_str_skip_mr,
    types::{Proof, U256},
};

pub struct TestProof {
    pub proof: Proof,
    pub public_inputs: Vec<U256>,
}

impl TestProof {
    pub fn new(proof: Proof, public_inputs: &[&str]) -> Self {
        TestProof {
            proof,
            public_inputs: public_inputs
                .iter()
                .map(|x| u256_from_str_skip_mr(x))
                .collect(),
        }
    }
}

/// Note: uses the [`super::vkey::TestVKey`] verification key
pub fn valid_proofs() -> Vec<TestProof> {
    vec![
        TestProof::new(
            proof_from_str_projective(
                (
                    "14690239631763315837453664042432597412358242015145136618358222387278279116195",
                    "3643780132787394650252740182203975834437718299044985767317449850565317488166",
                    "1",
                ),
                (
                    (
                        "12318858301116136039901780880140636659938620239898996708075490787377990627021",
                        "2655335215981242007154487245887430969280221036621749020134517693786655613279",
                    ),
                    (
                        "13665401110313137408934496500722861939604143361381592485089904000626841203657",
                        "16886134483886522029016161222749430345330639128944557054644673266184517343819",
                    ),
                    (
                        "1",
                        "0",
                    ),
                ),
                (
                    "20648835712776577082472214104799321681109444262412204126993043827327940209500",
                    "18221482463531702349023663967222567126976044483242847353303931705097934869008",
                    "1",
                ),
            ),
            &[
                "7889586699914970744657798935358222218486353295005298675075639741334684257960",
                "9606705614694883961284553030253534686862979817135488577431113592919470999200",
                "0",
                "0",
                "8028428639047162114812531350570986269919973729488816770273375429500049913662",
                "0",
                "0",
                "0",
                "120000",
                "1670075846",
                "12015639506942512288768672368651535943729197113218174743802158661212279174168",
                "0",
                "0",
                "241513166508321350627618709707967777063380694253583200648944705250489865558",
            ]
        ),
        TestProof::new(
            proof_from_str_projective(
                (
                    "7993009685331433638920395331150781889478566758995702966531973325559882244541",
                    "19377019684716159695405709376586094262600757371553814186267628013309634499679",
                    "1",
                ),
                (
                    (
                        "18294813972542074273163758181884905299738343873395476210048567332679083686962",
                        "12415589741393631617415988359584415987021178711928579059041575716011687648248",
                    ),
                    (
                        "15862404738956320094732459022428694815251563845574475032319287002192265570993",
                        "9747551887510890762693640119087480847766778714929202777532578357422174915815",
                    ),
                    (
                        "1",
                        "0",
                    ),
                ),
                (
                    "6110635641707836138291608269066893550836744326919704778091042044028598428274",
                    "2489843526990439173240146083067669570359846906943998533608630832291503210510",
                    "1",
                ),
            ),
            &[
                "7889586699914970744657798935358222218486353295005298675075639741334684257960",
                "9606705614694883961284553030253534686862979817135488577431113592919470999200",
                "3274987707755874055218761963679216380632837922347165546870932041376197622893",
                "21565952902710874749074047612627661909010394770856499168277361914501522149919",
                "18505238634407118839447741044834397583809065182892598442650259184768108193880",
                "0",
                "0",
                "0",
                "170000",
                "1670078279",
                "908158097066600914673776144051668000794530280731188389204488968169884520703",
                "0",
                "1",
                "241513166508321350627618709707967777063380694253583200648944705250489865558",
            ]
        ),
    ]
}

/// Note: uses the [`super::vkey::TestVKey`] verification key
pub fn invalid_proofs() -> Vec<TestProof> {
    vec![
        // Changed timestamp
        TestProof::new(
            proof_from_str_projective(
                (
                    "14690239631763315837453664042432597412358242015145136618358222387278279116195",
                    "3643780132787394650252740182203975834437718299044985767317449850565317488166",
                    "1",
                ),
                (
                    (
                        "12318858301116136039901780880140636659938620239898996708075490787377990627021",
                        "2655335215981242007154487245887430969280221036621749020134517693786655613279",
                    ),
                    (
                        "13665401110313137408934496500722861939604143361381592485089904000626841203657",
                        "16886134483886522029016161222749430345330639128944557054644673266184517343819",
                    ),
                    (
                        "1",
                        "0",
                    ),
                ),
                (
                    "20648835712776577082472214104799321681109444262412204126993043827327940209500",
                    "18221482463531702349023663967222567126976044483242847353303931705097934869008",
                    "1",
                ),
            ),
            &[
                "7889586699914970744657798935358222218486353295005298675075639741334684257960",
                "9606705614694883961284553030253534686862979817135488577431113592919470999200",
                "0",
                "0",
                "8028428639047162114812531350570986269919973729488816770273375429500049913662",
                "0",
                "0",
                "0",
                "120000",
                "1670075847",
                "12015639506942512288768672368651535943729197113218174743802158661212279174168",
                "0",
                "0",
                "241513166508321350627618709707967777063380694253583200648944705250489865558",
            ]
        ),

        // Changes A.x by one bit
        TestProof::new(
            proof_from_str_projective(
                (
                    "7993009685331433638920395331150781889478566758995702966531973325559882244540",
                    "19377019684716159695405709376586094262600757371553814186267628013309634499679",
                    "1",
                ),
                (
                    (
                        "18294813972542074273163758181884905299738343873395476210048567332679083686962",
                        "12415589741393631617415988359584415987021178711928579059041575716011687648248",
                    ),
                    (
                        "15862404738956320094732459022428694815251563845574475032319287002192265570993",
                        "9747551887510890762693640119087480847766778714929202777532578357422174915815",
                    ),
                    (
                        "1",
                        "0",
                    ),
                ),
                (
                    "6110635641707836138291608269066893550836744326919704778091042044028598428274",
                    "2489843526990439173240146083067669570359846906943998533608630832291503210510",
                    "1",
                ),
            ),
            &[
                "7889586699914970744657798935358222218486353295005298675075639741334684257960",
                "9606705614694883961284553030253534686862979817135488577431113592919470999200",
                "3274987707755874055218761963679216380632837922347165546870932041376197622893",
                "21565952902710874749074047612627661909010394770856499168277361914501522149919",
                "18505238634407118839447741044834397583809065182892598442650259184768108193880",
                "0",
                "0",
                "0",
                "170000",
                "1670078279",
                "908158097066600914673776144051668000794530280731188389204488968169884520703",
                "0",
                "1",
                "241513166508321350627618709707967777063380694253583200648944705250489865558",
            ]
        ),

        // Changes C to be the point at infinity
        TestProof::new(
            proof_from_str_projective(
                (
                    "7993009685331433638920395331150781889478566758995702966531973325559882244541",
                    "19377019684716159695405709376586094262600757371553814186267628013309634499679",
                    "1",
                ),
                (
                    (
                        "18294813972542074273163758181884905299738343873395476210048567332679083686962",
                        "12415589741393631617415988359584415987021178711928579059041575716011687648248",
                    ),
                    (
                        "15862404738956320094732459022428694815251563845574475032319287002192265570993",
                        "9747551887510890762693640119087480847766778714929202777532578357422174915815",
                    ),
                    (
                        "1",
                        "0",
                    ),
                ),
                (
                    "6110635641707836138291608269066893550836744326919704778091042044028598428274",
                    "2489843526990439173240146083067669570359846906943998533608630832291503210510",
                    "0",
                ),
            ),
            &[
                "7889586699914970744657798935358222218486353295005298675075639741334684257960",
                "9606705614694883961284553030253534686862979817135488577431113592919470999200",
                "3274987707755874055218761963679216380632837922347165546870932041376197622893",
                "21565952902710874749074047612627661909010394770856499168277361914501522149919",
                "18505238634407118839447741044834397583809065182892598442650259184768108193880",
                "0",
                "0",
                "0",
                "170000",
                "1670078279",
                "908158097066600914673776144051668000794530280731188389204488968169884520703",
                "0",
                "1",
                "241513166508321350627618709707967777063380694253583200648944705250489865558",
            ]
        ),
    ]
}
