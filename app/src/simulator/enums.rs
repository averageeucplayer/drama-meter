
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum BerserkerSkills {
    Whirlwind = 16070,
    Overdrive = 16640,
    TempestSlash = 16190,
    BraveSlash = 16630,
    BloodyRush = 16140,
    HellBlade = 16080,
    SwordStorm = 16600,
    RedDust = 16120,
    MountainCrash = 16220,
    FinishStrike = 16300,
    ShoulderCharge = 16060,
    AssaultBlade = 16110,
    PowerBreak = 16030,
    /// Identity
    BloodySurge = 16720,
    /// Hyper Awakening Technique
    BloodSlash = 16660,
    /// Hyper Awakening
    FuryMethod = 16650,
    /// Hyper Awakening Technique
    BerserkFury = 16710,
    /// Hyper Awakening
    RageDeathblade = 16730,
}

pub enum BerserkerBuffSkills {
    RedDustAtkPower = 161201
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum BardSkills {
    SerenadeOfCourage5 = 21140,
    SerenadeOfCourage10 = 21141,
    SerenadeOfCourage15 = 21142,
    Sonatina = 21290,
    WindOfMusic = 21070,
    WindOfMusicChain = 21079,
    Stigma = 21090,
    GuardianTune = 21250,
    PreludeOfStorm = 21080,
    RhapsodyOfLight = 21260,
    HeavenlyTune = 21160,
    SonicVibration = 21170,
    /// Hyper Awakening Technique
    Aria = 21300,
    /// Awakening
    Concerto = 21330,
    /// Awakening
    Symphonia = 21230,
    /// Hyper Awakening
    SymphonyMelody = 21320
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum BardSkillBuffs {
    IntenseTune = 211606,
    SonicVibrationAtkPower = 211749,
    HeavenlyTuneManaRegen = 211601,
    SonicVibrationManaRegen = 211767,
    SoundShockNoteBrand = 210230,
    SonatinaNoteBrand = 212906,
    AriaHyperAwakeningSkillDamage = 212306,
    AriaOutgoingDamage = 212305,
    SerenadeOfCourage5 = 211400,
    SerenadeOfCourage10 = 211410,
    SerenadeOfCourage15 = 211420,
    GuardianTuneDamageReduction = 212500,
    GuardianTuneDamageShield = 212513
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum PaladinSkills {
    AlithanesJudgment = 36210,
    AlithanesDevotion = 36230,
    HolyArea = 36120,
    HolyAura = 36800,
    AlithanesRage = 36240,
    LightShock = 36050,
    SwordOfJustice = 36080,
    GodsDecree = 36150,
    HolyExplosion = 36100,
    HeavenlyBlessings = 36200,
    WrathOfGod = 36170,
    DivineJustice = 36260
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum ArtistSkills {
    StrokeHopper = 31210,
    PaintSunsketch = 31400,
    PaintSunWell = 31410,
    PaintStarryNight = 31450,
    PaintDrawingOrchids = 31420,
    PaintIllusionDoor = 31220,
    HolyBeastSummonPhoenix = 31920,
    Moonfall = 31050,
    PaintCattleDrive = 31940,
    PaintDragonEngraving = 31950,
    MasterworkEfflorescence = 31910,
    DreamBlossomGarden = 31930,
    
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum SorceressSkills {
    Explosion = 37330,
    Doomsday = 37350,
    PunishingStrike = 37270
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum SouleaterSkills {
    LethalSpinning = 46250,
    DeadlyCombination = 46620,
    Fatality = 46630
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum DestroyerSkills {
    EarthWave = 18090,
    GalaxyBreak = 18240,
    HyperBigBang = 18250
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum GunlancerSkills {
    GuardiansOath = 17250,
    JusticeServed = 17260,
    SurgeCannon = 17200,
    ChargedStinger = 17210,
    GuardiansThundercrack = 17140
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum SlayerSkills {
    RageSlasher = 45820,
    RagnaDeathblade = 45830,
    Bloodlust = 45004
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum ArcanistSkills {
    Death = 19370,
    TheTower = 19360,
    Emperor = 19282
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum SummonerSkills {
    JudgeKelsion = 20350,
    BagronsFrenzy = 20370
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum WardancerSkills {
    UltimateSkillGreatRagingDemonKick = 22370,
    UltimateSkillEightTrigramsChaoticStrike = 22360,
    EsotericSkillAzureDragonSupremeFist = 22340
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum ScrapperSkills {
    DivineDragonCreation = 23410,
    SupremeHeavenShatteringFist = 23400,
    IronCannonBlow = 23230
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum SoulfistSkills {
    FallingSun = 24300,
    SupernovaPurgationRay = 24310,
    Shadowbreaker = 24200
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum GlaivierSkills {
    YeonStyleSpearTechniqueGalaxyFlyingSpear = 34620,
    YeonStyleSpearTechniqueDragonCavalryUnitySlash = 34630,
    RedDragonsHorn = 34590
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum StrikerSkills {
    UltimateSkillThunderboltKick = 39340,
    UltimateSkillMountainLordsExplosiveRoar = 39350,
    EsotericSkillCallOfTheWindGod = 39110
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum BreakerSkills {
    HeavenlyPunishment = 47300,
    CelestialFist = 47310,
    AsuraDestructionBasicAttack = 47020
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum DeathbladeSkills {
    EternalFlash = 25410,
    ChaoticDeathblade = 25420,
    Zero = 25038
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum ShadowhunterSkills {
    DarknessBlast = 27910,
    RayOfRuin = 27920,
    BloodMassacre = 27860
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum ReaperSkills {
    CadenzaDeLaLuna = 26940,
    RequiemDelSol = 26950
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum SharpshooterSkills {
    GigantarBowFenrir = 28260,
    AAGADeadeye = 28270
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum DeadeyeSkills {
    DeadlyCage = 29360,
    BlauerBlitz = 29370,
    JudgmentDay = 29300
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum ArtilleristSkills {
    ACOMBombardmentSupport = 30320,
    ACOMAttack = 30330,
    BarrageFocusFire = 30260
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum MachinistSkills {
    BattleshipOperation = 35810,
    AirStrike = 35930,
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum GunslingerSkills {
    DeadEnd = 38320,
    AtomicExplosion = 38330,
    Sharpshooter = 38110
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum AeromancerSkills {
    AkashasWave = 32290,
    KahnsTerritory = 32300,
    WindGimlet = 32250,
    PiercingWind = 32260
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum WildsoulSkills {
    ForbiddenSorceryRippingBear = 33400,
    ForbiddenSorceryFoxStarRainstorm = 33410,
    SmackSmite = 33520,
    FoxFireDance = 33530,
}