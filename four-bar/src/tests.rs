use crate::*;
use indicatif::ProgressBar;
use metaheuristics_nature::Report;
use plotters::prelude::*;
use std::{f64::consts::TAU, path::Path};

#[test]
fn planar() {
    // let target = Mechanism::four_bar(FourBar {
    //     p0: (0., 0.),
    //     a: 0.,
    //     l0: 90.,
    //     l1: 35.,
    //     l2: 70.,
    //     l3: 70.,
    //     // l4: 40.,
    //     // g: 0.5052948926891512,
    //     // l4: 84.7387,
    //     // g: 0.279854818911,
    //     l4: 77.0875,
    //     g: 5.88785793416,
    //     /////
    //     // NKhan 1
    //     // l0: 2.9587,
    //     // l1: 1.,
    //     // l2: 3.4723,
    //     // l3: 3.5771,
    //     // l4: 3.3454,
    //     // g: 3.3771,
    //     // NKhan 2
    //     // l0: 3.,
    //     // l1: 1.,
    //     // l2: 3.,
    //     // l3: 2.5,
    //     // l4: 1.,
    //     // g: 5.,
    // })
    // .four_bar_loop(TAU / 6., 360);
    // let target = YU1;
    // let target = YU2;
    // let target = PATH_HAND;
    let target = OPEN_CURVE2;
    let gen = 40;
    let pb = ProgressBar::new(gen as u64);
    let (ans, history) = synthesis::synthesis(&target, gen, 200, |r| {
        pb.set_position(r.gen as u64);
        true
    });
    pb.finish();
    let path = Mechanism::four_bar(ans).four_bar_loop(0., 360);
    plot_curve(
        "Synthesis Test",
        &[
            ("Target", &target, (221, 51, 85)),
            ("Optimized", &path, (118, 182, 222)),
        ],
        "result.svg",
    );
    plot_history(&history, "history.svg");
}

pub fn plot_curve<'a, S, P>(title: &str, curves: &[(S, &[[f64; 2]], (u8, u8, u8))], path: P)
where
    S: ToString + Copy,
    P: AsRef<Path>,
{
    const FONT: &str = if cfg!(windows) {
        "Times New Roman"
    } else {
        "Nimbus Roman No9 L"
    };
    let mut p_max = 0.;
    let mut p_min = f64::INFINITY;
    for (_, curve, _) in curves.iter() {
        let max = curve
            .iter()
            .fold(-f64::INFINITY, |v, &[x, y]| v.max(x.max(y)));
        let min = curve
            .iter()
            .fold(f64::INFINITY, |v, &[x, y]| v.min(x.min(y)));
        if max > p_max {
            p_max = max;
        }
        if min < p_min {
            p_min = min;
        }
    }
    let root = SVGBackend::new(&path, (1000, 1000)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(title, (FONT, 40))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .margin(20)
        .build_cartesian_2d(p_min..p_max, p_min..p_max)
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style((FONT, 20))
        .axis_desc_style((FONT, 20))
        .draw()
        .unwrap();
    for (i, &(name, curve, (r, g, b))) in curves.iter().enumerate() {
        let color = RGBColor(r, g, b);
        chart
            .draw_series(LineSeries::new(
                curve.iter().map(|&[x, y]| (x, y)),
                color.stroke_width(2),
            ))
            .unwrap()
            .label(name.to_string())
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });
        chart
            .draw_series(curve.iter().map(|&[x, y]| {
                if i % 2 == 1 {
                    Circle::new((x, y), 5, color.stroke_width(1)).into_dyn()
                } else {
                    TriangleMarker::new((x, y), 7, color.stroke_width(1)).into_dyn()
                }
            }))
            .unwrap();
    }
    chart
        .configure_series_labels()
        .label_font((FONT, 30))
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

pub fn plot_history<P>(history: &[Report], path: P)
where
    P: AsRef<Path>,
{
    let root = SVGBackend::new(&path, (1600, 900)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("History", ("sans-serif", 50))
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(20)
        .build_cartesian_2d(
            0..history.iter().map(|r| r.gen).max().unwrap(),
            0f64..history.iter().map(|r| r.best_f).fold(0., |a, b| b.max(a)),
        )
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Generation")
        .y_desc("Fitness")
        .draw()
        .unwrap();
    chart
        .draw_series(LineSeries::new(
            history.iter().map(|r| (r.gen, r.best_f)),
            RGBColor(118, 182, 222).stroke_width(5),
        ))
        .unwrap()
        .label("History")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RGBColor(118, 182, 222)));
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

const PATH_HAND: &[[f64; 2]] = &[
    [107.31911969201228, 81.59276839878613],
    [107.2463224148719, 82.21336703713774],
    [107.09767748992344, 82.8096978257258],
    [106.87350662227556, 83.37712932129989],
    [106.57507284914166, 83.91136896339214],
    [106.20456015538429, 84.40854784369684],
    [105.76502948370728, 84.86529396169908],
    [105.26035235410475, 85.27879204999705],
    [104.69512402388091, 85.64682849039397],
    [104.07455877255757, 85.96782032041233],
    [103.40437046207884, 86.24082783272775],
    [102.69064198180553, 86.4655507798514],
    [101.93968752370878, 86.64230869597502],
    [101.15791183425222, 86.7720063206304],
    [100.35167064898712, 86.8560855393411],
    [99.52713643236567, 86.8964656311437],
    [98.69017332242603, 86.89547392032281],
    [97.84622482663502, 86.85576916111027],
    [97.00021734483698, 86.7802601334607],
    [96.15648202569761, 86.67202199234208],
    [95.31869681551282, 86.53421289233917],
    [94.48984985671751, 86.36999330679299],
    [93.67222466359516, 86.18245028203907],
    [92.8674067710462, 85.9745286209323],
    [92.07631084508772, 85.74897068629815],
    [91.2992265860777, 85.50826616650141],
    [90.5358811703725, 85.2546127654966],
    [89.78551548312964, 84.9898883827541],
    [89.0469710104255, 84.71563494877088],
    [88.31878399465106, 84.43305369354597],
    [87.59928332046609, 84.14301126164631],
    [86.88668859173009, 83.84605576021451],
    [86.17920498016159, 83.54244154565382],
    [85.47511166665717, 83.2321613289109],
    [84.77284104447227, 82.91498401411799],
    [84.0710462942516, 82.59049658427077],
    [83.36865545539672, 82.25814831154912],
    [82.66491068529979, 81.91729559732507],
    [81.95939199482362, 81.56724583403634],
    [81.25202535172939, 81.20729882203862],
    [80.54307563049495, 80.83678446158672],
    [79.83312543528132, 80.45509566410531],
    [79.12304131287203, 80.06171567771193],
    [78.41392928717721, 79.65623928875132],
    [77.70708197268509, 79.23838763291982],
    [77.00391975123452, 78.80801661566994],
    [76.305928618995, 78.36511919192738],
    [75.61459732717438, 77.90982198068507],
    [74.9313563535751, 77.44237688307152],
    [74.25752105953276, 76.96314852693622],
    [73.59424111851, 76.47259847254509],
    [72.94245796230254, 75.9712671802288],
    [72.30287159458089, 75.45975476128268],
    [71.67591768723906, 74.93870150946367],
    [71.0617554216584, 74.4087691452031],
    [70.46026608361711, 73.87062360284901],
    [69.87106198569316, 73.32492005888345],
    [69.29350489177783, 72.77229074316723],
    [68.72673276988193, 72.2133359035938],
    [68.16969341431452, 71.64861811518323],
    [67.62118326604734, 71.07865994574458],
    [67.07988962676134, 70.50394481957197],
    [66.5444344103151, 69.92492076538834],
    [66.01341760422352, 69.34200660116508],
    [65.4854587188504, 68.75560000163951],
    [64.95923467592608, 68.16608681812343],
    [64.43351282052636, 67.57385097690097],
    [63.90717801941475, 66.97928427301918],
    [63.37925311967692, 66.38279539996236],
    [62.84891236995029, 65.78481761053723],
    [62.31548773708055, 65.18581448695727],
    [61.77846836894092, 64.58628340416703],
    [61.23749374570035, 63.98675639455843],
    [60.69234131491437, 63.3877982584224],
    [60.14290961045745, 62.79000190639724],
    [59.5891980040459, 62.19398106135909],
    [59.03128432621916, 61.60036058137255],
    [58.469301619383174, 61.009764786628786],
    [57.90341525001237, 60.42280427657185],
    [57.3338015142978, 59.840061804340614],
    [56.76062872790151, 59.262077830967755],
    [56.18404160469575, 58.6893364093502],
    [55.60414951182473, 58.122252046922135],
    [55.021018950738004, 57.56115816653816],
    [54.43467036824567, 57.006297728795374],
    [53.8450791604491, 56.45781649849144],
    [53.2521805073781, 55.915759336686556],
    [52.655877478039564, 55.38006978227936],
    [52.056051683516046, 54.85059305809291],
    [51.45257563696147, 54.327082501563424],
    [50.84532590877475, 53.809209284762],
    [50.23419614539209, 53.29657515813417],
    [49.61910905097875, 52.78872783219855],
    [49.00002651029305, 52.285178506215026],
    [48.376957153262886, 51.78542096655504],
    [47.74996082042011, 51.28895161341555],
    [47.1191495746234, 50.795289734921205],
    [46.4846851085448, 50.30399733387347],
    [45.84677260848657, 49.81469782471648],
    [45.205651342275914, 49.327092955972944],
    [44.561582431570976, 48.84097737476783],
    [43.91483443701089, 48.35625033253587],
    [43.265667519638576, 47.8729241312742],
    [42.614317036971975, 47.391129023819296],
    [41.96097748207152, 46.91111440521276],
    [41.30578767625485, 46.43324626062317],
    [40.64881808042783, 45.958000963779156],
    [39.99006099838817, 45.48595564379846],
    [39.329424312256606, 45.017775453310136],
    [38.66672922180632, 44.55419817290814],
    [38.00171226406979, 44.096016672872885],
    [37.33403167674908, 43.64405982003248],
    [36.663277949107275, 43.199172463646995],
    [35.988988188057284, 42.76219515812577],
    [35.310663725874534, 42.33394428187448],
    [34.62779021950146, 41.91519319107542],
    [33.9398593488435, 41.506655005936054],
    [33.24639112027922, 41.10896756679949],
    [32.54695572742067, 40.72268102098722],
    [31.841193917348168, 40.348248411316085],
    [31.128834858135182, 39.98601953723402],
    [30.409710601039777, 39.636238253016664],
    [29.683766374448247, 39.29904325811366],
    [28.95106613044038, 38.974472326150526],
    [28.2117929806532, 38.66246981473335],
    [27.46624439627075, 38.36289720124969],
    [26.714822296591038, 38.07554630312585],
    [25.958018400139995, 37.80015476684654],
    [25.1963954499746, 37.53642335032626],
    [24.430565139261002, 37.28403447925699],
    [23.66116374395195, 37.04267153059621],
    [22.888826607324553, 36.812038285603336],
    [22.11416270901415, 36.59187800045423],
    [21.33773058389827, 36.381991563636035],
    [20.560016831101294, 36.18225424479929],
    [19.781418370441095, 35.99263058789167],
    [19.002229465394315, 35.81318706029654],
    [18.222634343223824, 35.64410213721319],
    [17.44270601175004, 35.48567357436508],
    [16.662411607857322, 35.33832269997143],
    [15.88162432635373, 35.2025956364442],
    [15.100141681559002, 35.079161441237375],
    [14.31770956093311, 34.96880723257924],
    [13.534051253193013, 34.87243043755642],
    [12.748900385211428, 34.791028365519196],
    [11.962036493998276, 34.72568536762677],
    [11.173321802100112, 34.677557892424026],
    [10.382737664649596, 34.647857786800344],
    [9.590419119494953, 34.63783422098178],
    [8.796686001102032, 34.6487546350957],
    [8.002069174208394, 34.681885113336925],
    [7.207330601620605, 34.73847059012965],
    [6.4134761764394455, 34.81971528142214],
    [5.6217605141852545, 34.92676371408341],
    [4.833683204379518, 35.06068269818154],
    [4.050976351932803, 35.22244455175337],
    [3.2755835827200883, 35.412911846685084],
    [2.509631030806595, 35.632823898770646],
    [1.7553911526040054, 35.88278517621289],
    [1.0152405119721166, 36.163255750137836],
    [0.29161293718626524, 36.474543859463054],
    [-0.4130503453434642, 36.81680061203541],
    [-1.0963518536931645, 37.19001679562145],
    [-1.7559884488912019, 37.59402172730927],
    [-2.38979956727826, 38.02848402927675],
    [-2.9958113362055983, 38.492914183678955],
    [-3.5722749644627143, 38.986668690439046],
    [-4.11769800608365, 39.50895562964505],
    [-4.630867360773571, 40.058841415515026],
    [-5.110863185729826, 40.63525852173899],
    [-5.557063239722119, 41.237013958447974],
    [-5.969137547079875, 41.86279828887762],
    [-6.347033641940001, 42.511194988524345],
    [-6.690953016667258, 43.18068997053983],
    [-7.0013197380424685, 43.86968112736116],
    [-7.278742496829345, 44.57648776901267],
    [-7.523971608416687, 45.29935987185644],
    [-7.7378526741527125, 46.036487086392754],
    [-7.921278736967324, 46.786007487513054],
    [-8.075142815921069, 47.54601608384178],
    [-8.200292680420127, 48.31457313294641],
    [-8.297489627052464, 49.08971233480354],
    [-8.367372854414178, 49.86944899567473],
    [-8.410430800831278, 50.65178826736886],
    [-8.42698052601856, 51.434733571892544],
    [-8.417155892085681, 52.21629531817127],
    [-8.380904945196491, 52.99450000565737],
    [-8.317996531014664, 53.76739978938862],
    [-8.228035809739133, 54.53308255297842],
    [-8.11048798484645, 55.28968250103329],
    [-7.964709237688069, 56.035391241907185],
    [-7.789983580614191, 56.768469287146644],
    [-7.585564115258805, 57.48725784736303],
    [-7.350717018707876, 58.19019075771814],
    [-7.084766484558749, 58.87580632198723],
    [-6.787138821594624, 59.54275882457424],
    [-6.457403960225143, 60.18982942715674],
    [-6.095312733321109, 60.81593614292966],
    [-5.700828478134653, 61.420142568546595],
    [-5.274151741634903, 62.00166505332111],
    [-4.81573715259956, 62.559877998106266],
    [-4.326301838236887, 63.09431700306658],
    [-3.806825097802232, 63.60467962427745],
    [-3.258539386778743, 64.0908235531145],
    [-2.6829129988065787, 64.55276209850274],
    [-2.081625145259416, 64.99065692846227],
    [-1.4565344118419503, 65.40480811164011],
    [-0.8096418070347724, 65.79564158879472],
    [-0.14304979986399502, 66.16369429525469],
    [0.5410811321228408, 66.50959724466976],
    [1.2405768635066607, 66.834056968243],
    [1.953293313104119, 67.13783577839463],
    [2.677157957114609, 67.42173138794574],
    [3.410207778256378, 67.68655646220833],
    [4.150622443310731, 67.93311870907351],
    [4.896751722493391, 68.16220211915135],
    [5.647136418366102, 68.37454995279542],
    [6.400522348169979, 68.57085003282712],
    [7.155867209401173, 68.75172284122327],
    [7.912340441913813, 68.9177128361297],
    [8.669316468970656, 69.0692833044138],
    [9.426361943502329, 69.20681494754291],
    [10.183217834780166, 69.33060826864613],
    [10.939777356890573, 69.44088969063509],
    [11.696060858020271, 69.5378211942092],
    [12.452188855087932, 69.6215131257935],
    [13.208354410533058, 69.69203969444426],
    [13.964796008323361, 69.74945655894832],
    [14.721771997971011, 69.79381980688565],
    [15.479537544126746, 69.8252055509765],
    [16.238324852499947, 69.8437293185345],
    [16.99832724919259, 69.84956439034114],
    [17.75968747976963, 69.8429582577297],
    [18.522490376781256, 69.82424641192503],
    [19.286759830288858, 69.7938627572731],
    [20.052459795090087, 69.75234604816536],
    [20.819498889757952, 69.70034188519394],
    [21.58773799399888, 69.63859996514208],
    [22.357000138275556, 69.56796645651619],
    [23.127081907340052, 69.48937156124471],
    [23.897765549468826, 69.40381251699708],
    [24.66883099586894, 69.31233248594897],
    [25.440067047966235, 69.21599595721058],
    [26.211281080186374, 69.11586145412485],
    [26.98230672679328, 69.01295247723864],
    [27.753009166327505, 68.90822772266048],
    [28.523287778121322, 68.80255168843428],
    [29.29307611353792, 68.69666681439051],
    [30.06233929106149, 68.59116829101731],
    [30.831069080459926, 68.48648261913488],
    [31.599277078904905, 68.38285090513041],
    [32.366986494165374, 68.2803177384914],
    [33.134223131170565, 68.17872632329932],
    [33.901006224384545, 68.0777203287036],
    [34.66733976740362, 67.97675269207316],
    [35.43320496275564, 67.8751013605671],
    [36.19855435074847, 67.77189170120461],
    [36.96330807992513, 67.66612505563988],
    [37.727352658418354, 67.5567126734547],
    [38.490542381837955, 67.44251403640202],
    [39.25270347690362, 67.32237839465735],
    [40.01364083916687, 67.19518818284766],
    [40.77314708641997, 67.0599028752732],
    [41.531013505173405, 66.9156017816184],
    [42.2870423437122, 66.76152428008314],
    [43.04105980854382, 66.59710603580452],
    [43.792929057030314, 66.42200985816226],
    [44.542562451534714, 66.23615000843226],
    [45.28993235156762, 66.03970897460036],
    [46.035079770313914, 65.83314597635984],
    [46.77812030869993, 65.61719674208605],
    [47.51924690008759, 65.39286440116821],
    [48.25872904627241, 65.16140164865838],
    [48.99690839376331, 64.92428465326428],
    [49.73419068021591, 64.68317948248972],
    [50.47103426549447, 64.43990209863799],
    [51.20793564091565, 64.19637322549242],
    [51.945412474660806, 63.954569587907386],
    [52.68398489255006, 63.71647317688573],
    [53.42415580377098, 63.48402028441791],
    [54.166391154520575, 63.25905208095827],
    [54.911101024330904, 63.04326847178464],
    [55.65862246753206, 62.83818686695121],
    [56.40920494538937, 62.64510733589627],
    [57.162999094641314, 62.465085397228954],
    [57.920049439283005, 62.298913424262835],
    [58.68029148031376, 62.14711133697259],
    [59.44355340033904, 62.009926912383975],
    [60.209562405379344, 61.88734569039701],
    [60.97795550496856, 61.77911009394179],
    [61.74829431417348, 61.684747034737356],
    [62.52008325814966, 61.60360295214337],
    [63.29279038143687, 61.534884945306345],
    [64.06586981962896, 61.47770641945052],
    [64.8387848881801, 61.431135485504605],
    [65.61103068797489, 61.39424423598143],
    [66.38215512382274, 61.36615697442045],
    [67.15177728179938, 61.346095503409146],
    [67.91960221342191, 61.333419677084805],
    [68.6854313255783, 61.327661595133556],
    [69.44916776909265, 61.3285520509606],
    [70.2108164477571, 61.3360381386585],
    [70.9704785236929, 61.35029126112557],
    [71.72834056264335, 61.37170515275931],
    [72.48465873190875, 61.40088392069586],
    [73.23973872135281, 61.438620503779575],
    [73.99391229162427, 61.485866333135434],
    [74.74751155160058, 61.543693337401585],
    [75.50084221850554, 61.61324975514977],
    [76.25415721047551, 61.69571148391075],
    [77.00763195609349, 61.79223089850239],
    [77.76134277479579, 61.90388520230271],
    [78.51524958515486, 62.03162642763949],
    [79.26918403696922, 62.17623517243125],
    [80.02284394294269, 62.33828004953639],
    [80.77579461448904, 62.51808463599298],
    [81.52747739440201, 62.71570344754915],
    [82.27722533954449, 62.93090813852436],
    [83.02428565376528, 63.16318474958008],
    [83.7678481204889, 63.41174241001812],
    [84.50707845182383, 63.675533462017206],
    [85.24115517232312, 63.95328452808764],
    [85.96930840550354, 64.2435376067813],
    [86.69085874303397, 64.54469987198406],
    [87.40525426107183, 64.85510048380712],
    [88.11210371367736, 65.17305240865679],
    [88.81120398449096, 65.49691700501445],
    [89.50256001629128, 65.82516896988508],
    [90.18639566135927, 66.15645916599965],
    [90.8631541977384, 66.48967286578029],
    [91.53348762793559, 66.82398105555222],
    [92.19823430452557, 67.1588826398957],
    [92.85838489587182, 67.49423566542542],
    [93.51503719688586, 67.83027603659848],
    [94.16934078500037, 68.16762261149486],
    [94.82243300011679, 68.50726802860508],
    [95.47536816902465, 68.85055511032033],
    [96.12904238035665, 69.19913919757366],
    [96.78411642788777, 69.55493727477523],
    [97.44093976270038, 69.92006522665848],
    [98.09947841632746, 70.29676501140855],
    [98.75924986904903, 70.687323921275],
    [99.41926773576726, 71.09398841848292],
    [100.07799892642112, 71.51887526877529],
    [100.73333561331191, 71.96388283835374],
    [101.3825839129856, 72.43060546654608],
    [102.02247067859214, 72.92025377385951],
    [102.64916921676826, 73.43358361434883],
    [103.25834411103472, 73.97083613709127],
    [103.84521467385079, 74.53169109199689],
    [104.4046358857587, 75.11523511121345],
    [104.93119503711094, 75.71994623267622],
    [105.41932169001964, 76.34369542271531],
    [105.8634080485119, 76.98376531747189],
    [106.25793638436835, 77.63688585653549],
    [106.59760983273677, 78.29928594535215],
    [106.87748265959081, 78.96675977384264],
    [107.09308602237105, 79.63474595456469],
    [107.24054530086866, 80.29841724027493],
    [107.31668526773566, 80.95277825132874],
];

const YU1: &[[f64; 2]] = &[
    [-27., 1.],
    [-21.857, -3.214],
    [-16.7, -7.428],
    [-11.571, -11.642],
    [-6.428, -15.857],
    [-1.285, -20.071],
    [3.857, -24.285],
    [9., -28.5],
    [15., -29.9],
    [20., -30.],
    [27.2, -25.],
    [29.2, -20.],
    [28., -10.],
    [22.7, 2.],
    [15., 10.6],
    [5., 16.5],
    [-10., 19.6],
    [-22., 17.],
    [-28., 11.],
    [-29., 5.],
];

const YU2: &[[f64; 2]] = &[
    [-24., 40.],
    [-30., 41.],
    [-34., 40.],
    [-38., 36.],
    [-36., 30.],
    [-28., 29.],
    [-21., 31.],
    [-17., 32.],
    [-8., 34.],
    [3., 37.],
    [10., 41.],
    [17., 41.],
    [26., 39.],
    [28., 33.],
    [29., 26.],
    [26., 23.],
    [17., 23.],
    [11., 24.],
    [6., 27.],
    [0., 31.],
];

const OPEN_CURVE: &[[f64; 2]] = &[
    [0.028607755880487345, 47.07692307692308],
    [6.182453909726641, 52.76923076923077],
    [14.797838525111256, 57.07692307692308],
    [24.643992371265103, 58.61538461538461],
    [41.10553083280357, 59.07692307692308],
    [50.18245390972664, 56.76923076923077],
    [60.6439923712651, 51.53846153846154],
    [65.41322314049587, 46.0],
    [68.79783852511126, 36.92307692307692],
    [67.41322314049587, 25.384615384615383],
    [60.6439923712651, 18.153846153846153],
];

const OPEN_CURVE2: &[[f64; 2]] = &[
    [-11.815980048813604, 42.67460370622394],
    [-7.873327719064499, 47.692524853177346],
    [2.520937513910411, 54.144137756403154],
    [14.528105972691773, 58.80363596428845],
    [20.26287299778138, 69.37711266679742],
    [24.922371205666682, 80.48822377790853],
    [30.65713823075629, 88.91116284600889],
    [39.2592887683907, 98.40937073131354],
    [52.16251457484231, 103.9649262868691],
    [60.22703070387457, 103.606503347801],
    [61.48151099061292, 99.6638510180519],
    [58.07649306946597, 96.43804456643899],
    [50.908034288103956, 90.7032775413494],
    [43.73957550674195, 86.40220227253218],
];
