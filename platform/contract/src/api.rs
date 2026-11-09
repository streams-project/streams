//! Every route of the service as data. OpenAPI is generated from this table, so no route exists
//! that the contract does not name.

/// Who may call a route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Auth {
    /// No token.
    Open,
    /// A member's bearer token.
    Member,
    /// A member's bearer token on a session with an active profile.
    Profile,
    /// The token of a hosted device.
    Device,
    /// The service token between server and finder.
    Service,
}

/// The group a route belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Group {
    Start,
    Auth,
    Account,
    Profiles,
    Pages,
    Commands,
    Play,
    Finder,
}

/// One query parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Query {
    pub name: &'static str,
    pub required: bool,
    pub about: &'static str,
}

/// One route. Path parameters are read from the path. Names are type names of this crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Route {
    pub method: &'static str,
    pub path: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub auth: Auth,
    pub group: Group,
    pub request: Option<&'static str>,
    pub response: &'static str,
    pub query: &'static [Query],
    /// True when a client may hold the answer and ask again with `If-None-Match`.
    pub cacheable: bool,
    /// A text media type this route may answer in place of JSON.
    pub text_form: Option<&'static str>,
}

/// Headers every request to the server carries, with what each holds.
pub const CLIENT_HEADERS: &[(&str, &str)] = &[
    ("Client-App", "The app name."),
    ("Client-Version", "The app version."),
    ("Client-Class", "The device class."),
    ("Client-Os", "The operating system."),
];

const fn read(
    path: &'static str,
    name: &'static str,
    about: &'static str,
    response: &'static str,
    access: (Auth, Group),
) -> Route {
    write("GET", path, name, about, None, response, access)
}

const fn write(
    method: &'static str,
    path: &'static str,
    name: &'static str,
    about: &'static str,
    request: Option<&'static str>,
    response: &'static str,
    (auth, group): (Auth, Group),
) -> Route {
    Route {
        method,
        path,
        name,
        about,
        auth,
        group,
        request,
        response,
        query: &[],
        cacheable: false,
        text_form: None,
    }
}

const fn cached(route: Route) -> Route {
    Route {
        cacheable: true,
        ..route
    }
}

const fn with_query(route: Route, query: &'static [Query]) -> Route {
    Route { query, ..route }
}

const fn with_text(route: Route, text_form: &'static str) -> Route {
    Route {
        text_form: Some(text_form),
        ..route
    }
}

/// True when `path` fits the `template`, whose `{name}` parts match one segment each.
fn fits(template: &str, path: &str) -> bool {
    let mut wanted = template.split('/');
    let mut given = path.split('/');
    loop {
        match (wanted.next(), given.next()) {
            (None, None) => return true,
            (Some(want), Some(have)) => {
                let any = want.starts_with('{') && !have.is_empty();
                if !any && want != have {
                    return false;
                }
            }
            _ => return false,
        }
    }
}

/// The route for a method and a path with no query.
#[must_use]
pub fn find(method: &str, path: &str) -> Option<&'static Route> {
    ROUTES
        .iter()
        .find(|route| route.method == method && fits(route.path, path))
}

const START: (Auth, Group) = (Auth::Open, Group::Start);
const LANGUAGE: &[Query] = &[Query {
    name: "lang",
    required: false,
    about: "A language code. Without it the text is in English.",
}];
const SIGN_IN: (Auth, Group) = (Auth::Open, Group::Auth);
const SIGNED_IN: (Auth, Group) = (Auth::Member, Group::Auth);
const ACCOUNT: (Auth, Group) = (Auth::Member, Group::Account);
const PROFILES: (Auth, Group) = (Auth::Member, Group::Profiles);
const WATCHING: (Auth, Group) = (Auth::Profile, Group::Profiles);
const PAGES: (Auth, Group) = (Auth::Profile, Group::Pages);
const COMMANDS: (Auth, Group) = (Auth::Profile, Group::Commands);
const PLAY: (Auth, Group) = (Auth::Profile, Group::Play);
const HOSTED: (Auth, Group) = (Auth::Device, Group::Play);
const REPORTS: (Auth, Group) = (Auth::Member, Group::Play);
const FINDER: (Auth, Group) = (Auth::Service, Group::Finder);
const HEALTH: (Auth, Group) = (Auth::Open, Group::Finder);

/// Every route the server and the finder answer.
pub const ROUTES: &[Route] = &[
    // Start
    cached(read(
        "/v1/hello",
        "getHello",
        "Features, limits and the contract range.",
        "Hello",
        START,
    )),
    read(
        "/v1/ready",
        "getReady",
        "Build, data hash, data age and part states.",
        "Ready",
        START,
    ),
    cached(read(
        "/v1/strings/{lang}",
        "getStrings",
        "Interface text for one language.",
        "Strings",
        START,
    )),
    with_query(
        cached(read(
            "/v1/help",
            "helpHome",
            "Help topics.",
            "HelpHome",
            START,
        )),
        LANGUAGE,
    ),
    with_query(
        cached(read(
            "/v1/help/articles/{slug}",
            "helpArticle",
            "One help article.",
            "HelpArticle",
            START,
        )),
        LANGUAGE,
    ),
    with_query(
        cached(read(
            "/v1/help/policies/{id}",
            "helpPolicy",
            "One policy.",
            "HelpPolicy",
            START,
        )),
        LANGUAGE,
    ),
    with_query(
        read(
            "/v1/help/search",
            "helpSearch",
            "Help articles holding every word.",
            "HelpResults",
            START,
        ),
        &[
            Query {
                name: "q",
                required: true,
                about: "The search text.",
            },
            Query {
                name: "lang",
                required: false,
                about: "A language code. Without it the text is in English.",
            },
        ],
    ),
    cached(read(
        "/v1/languages",
        "listLanguages",
        "Every language this server has text for.",
        "LanguageList",
        START,
    )),
    cached(read(
        "/v1/public/trending",
        "listPublicTrending",
        "Today's most watched titles for a visitor, with no profile: films and series in turn.",
        "PublicTitles",
        START,
    )),
    cached(read(
        "/v1/public/titles/{kind}/{id}",
        "getPublicTitle",
        "A title for a visitor, with no profile.",
        "TitleDetail",
        START,
    )),
    // Sign in
    write(
        "POST",
        "/v1/auth/password",
        "signInPassword",
        "Sign in with email and password.",
        Some("PasswordSignIn"),
        "SignedIn",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/code",
        "sendCode",
        "Send a code by email.",
        Some("CodeSend"),
        "CodeSent",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/code/verify",
        "signInCode",
        "Sign in with an emailed code.",
        Some("CodeSignIn"),
        "SignedIn",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/signup",
        "signUp",
        "Make an account.",
        Some("SignUp"),
        "SignedIn",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/refresh",
        "refresh",
        "Renew tokens. The refresh token rotates.",
        Some("RefreshRequest"),
        "SignedIn",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/signout",
        "signOut",
        "End this session, or every session.",
        Some("SignOut"),
        "Done",
        SIGNED_IN,
    ),
    write(
        "POST",
        "/v1/auth/confirm",
        "confirm",
        "Step up with a password or code.",
        Some("Confirm"),
        "Confirmed",
        SIGNED_IN,
    ),
    write(
        "POST",
        "/v1/auth/recovery/start",
        "recoveryStart",
        "Send a recovery code.",
        Some("RecoveryStart"),
        "Done",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/recovery/finish",
        "recoveryFinish",
        "Set a new password with a recovery code.",
        Some("RecoveryFinish"),
        "Done",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/device/code",
        "deviceCode",
        "Start a device sign in.",
        Some("DeviceCodeRequest"),
        "DeviceCode",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/device/token",
        "deviceToken",
        "Poll a device sign in.",
        Some("DeviceTokenRequest"),
        "DeviceTokenAnswer",
        SIGN_IN,
    ),
    write(
        "POST",
        "/v1/auth/device/approve",
        "deviceApprove",
        "Approve a device from a signed in screen.",
        Some("DeviceDecision"),
        "Done",
        SIGNED_IN,
    ),
    write(
        "POST",
        "/v1/auth/device/deny",
        "deviceDeny",
        "Refuse a device from a signed in screen.",
        Some("DeviceDecision"),
        "Done",
        SIGNED_IN,
    ),
    // Device and account
    write(
        "POST",
        "/v1/devices/me",
        "registerDevice",
        "Send the card hash, and the card when asked.",
        Some("DeviceCardRequest"),
        "DeviceCardAnswer",
        ACCOUNT,
    ),
    read(
        "/v1/devices",
        "listDevices",
        "Signed in devices.",
        "DeviceList",
        ACCOUNT,
    ),
    with_query(
        write(
            "DELETE",
            "/v1/devices",
            "removeDevices",
            "Sign out one device, or all others when no id is given.",
            None,
            "Done",
            ACCOUNT,
        ),
        &[Query {
            name: "id",
            required: false,
            about: "The device to sign out.",
        }],
    ),
    read(
        "/v1/account",
        "getAccount",
        "The account.",
        "Account",
        ACCOUNT,
    ),
    write(
        "PATCH",
        "/v1/account",
        "updateAccount",
        "Change account names.",
        Some("AccountUpdate"),
        "Account",
        ACCOUNT,
    ),
    write(
        "DELETE",
        "/v1/account",
        "deleteAccount",
        "Delete the account. Needs a step up.",
        None,
        "Done",
        ACCOUNT,
    ),
    read(
        "/v1/account/export",
        "exportAccount",
        "Everything held about the account.",
        "AccountExport",
        ACCOUNT,
    ),
    write(
        "POST",
        "/v1/account/password",
        "changePassword",
        "Change the password. Needs a step up.",
        Some("PasswordChange"),
        "Done",
        ACCOUNT,
    ),
    write(
        "POST",
        "/v1/account/email",
        "changeEmail",
        "Change the email. Needs a step up.",
        Some("EmailChange"),
        "Done",
        ACCOUNT,
    ),
    write(
        "POST",
        "/v1/account/phone",
        "changePhone",
        "Change the phone. Needs a step up.",
        Some("PhoneChange"),
        "Done",
        ACCOUNT,
    ),
    write(
        "PUT",
        "/v1/consent",
        "setConsent",
        "Set consent for this session.",
        Some("Consent"),
        "Consent",
        ACCOUNT,
    ),
    // Profiles
    read(
        "/v1/profiles",
        "listProfiles",
        "Profiles of the account.",
        "ProfileList",
        PROFILES,
    ),
    read(
        "/v1/profiles/gate",
        "profileGate",
        "Up to three titles over the profile chooser, each with a line of facts under its logo.",
        "ProfileGate",
        PROFILES,
    ),
    read(
        "/v1/profiles/{id}/feature",
        "profileFeature",
        "One title to show beside a profile on the picker.",
        "Tile",
        PROFILES,
    ),
    read(
        "/v1/profiles/{id}/avatars",
        "profileAvatars",
        "The pictures this profile can wear.",
        "AvatarSets",
        PROFILES,
    ),
    write(
        "POST",
        "/v1/profiles",
        "createProfile",
        "Add a profile.",
        Some("ProfileCreate"),
        "Profile",
        PROFILES,
    ),
    write(
        "PATCH",
        "/v1/profiles/{id}",
        "updateProfile",
        "Change a profile.",
        Some("ProfileUpdate"),
        "Profile",
        PROFILES,
    ),
    write(
        "DELETE",
        "/v1/profiles/{id}",
        "deleteProfile",
        "Delete a profile.",
        None,
        "Done",
        PROFILES,
    ),
    write(
        "POST",
        "/v1/profiles/{id}/enter",
        "enterProfile",
        "Write the active profile to this session.",
        Some("ProfileEnter"),
        "ProfileEntered",
        PROFILES,
    ),
    write(
        "PUT",
        "/v1/profiles/{id}/pin",
        "setPin",
        "Set or clear a PIN. Needs a step up.",
        Some("PinSet"),
        "Done",
        PROFILES,
    ),
    write(
        "POST",
        "/v1/profiles/{id}/main",
        "setMainProfile",
        "Make this the main profile.",
        None,
        "Done",
        PROFILES,
    ),
    write(
        "POST",
        "/v1/profiles/{id}/transfer",
        "transferProfile",
        "Move a profile to another account.",
        Some("ProfileTransfer"),
        "Done",
        PROFILES,
    ),
    read(
        "/v1/profiles/{id}/limits",
        "getLimits",
        "Age rating and blocked titles.",
        "ProfileLimits",
        PROFILES,
    ),
    write(
        "PUT",
        "/v1/profiles/{id}/limits",
        "setLimits",
        "Change age rating and blocked titles. Needs a step up.",
        Some("ProfileLimits"),
        "ProfileLimits",
        PROFILES,
    ),
    read(
        "/v1/profiles/me/settings",
        "getSettings",
        "Settings of the active profile.",
        "ProfileSettings",
        WATCHING,
    ),
    // Pages
    cached(read(
        "/v1/nav",
        "getNav",
        "The menu for this class and profile.",
        "Nav",
        PAGES,
    )),
    with_query(
        cached(read(
            "/v1/screens/{name}",
            "getScreen",
            "A built page.",
            "Screen",
            PAGES,
        )),
        &[Query {
            name: "genre",
            required: false,
            about: "The genre, for the genre screen.",
        }],
    ),
    read(
        "/v1/more/{cursor}",
        "getMore",
        "The next part of a screen or section.",
        "More",
        PAGES,
    ),
    cached(read(
        "/v1/titles/{kind}/{id}",
        "getTitle",
        "One title.",
        "TitleDetail",
        PAGES,
    )),
    cached(read(
        "/v1/titles/{kind}/{id}/seasons/{n}",
        "getSeason",
        "One season.",
        "Season",
        PAGES,
    )),
    with_query(
        read(
            "/v1/search",
            "search",
            "Search results as a screen.",
            "Screen",
            PAGES,
        ),
        &[Query {
            name: "q",
            required: true,
            about: "The search text.",
        }],
    ),
    with_query(
        read(
            "/v1/search/suggest",
            "suggest",
            "Search suggestions.",
            "Suggestions",
            PAGES,
        ),
        &[Query {
            name: "q",
            required: true,
            about: "The search text.",
        }],
    ),
    with_query(
        read(
            "/v1/notifications",
            "getNotifications",
            "The notification feed for this profile.",
            "Notifications",
            PAGES,
        ),
        &[Query {
            name: "cursor",
            required: false,
            about: "The cursor from a previous answer.",
        }],
    ),
    read(
        "/v1/notifications/count",
        "getNotificationCount",
        "Unread notification count for this profile.",
        "NotificationCount",
        PAGES,
    ),
    write(
        "POST",
        "/v1/notifications/read",
        "readNotifications",
        "Mark notifications as read up to an item id.",
        Some("NotificationRead"),
        "Done",
        PAGES,
    ),
    // Commands
    write(
        "POST",
        "/v1/commands",
        "sendCommands",
        "An idempotent batch of changes, answered per op.",
        Some("CommandBatch"),
        "CommandAnswer",
        COMMANDS,
    ),
    with_query(
        read(
            "/v1/changes",
            "getChanges",
            "Changes after a cursor.",
            "Changes",
            COMMANDS,
        ),
        &[Query {
            name: "cursor",
            required: false,
            about: "Where the last answer stopped.",
        }],
    ),
    // Play
    write(
        "POST",
        "/v1/play",
        "play",
        "Ask to play. Answers ranked attempts and where to start.",
        Some("PlayRequest"),
        "PlayAnswer",
        PLAY,
    ),
    write(
        "POST",
        "/v1/play/{id}/more",
        "playMore",
        "Attempts that arrived late.",
        None,
        "MoreAttempts",
        PLAY,
    ),
    write(
        "POST",
        "/v1/play/{id}/marks",
        "playMarks",
        "Skip marks, after first frame, series only.",
        Some("MarksRequest"),
        "Marks",
        PLAY,
    ),
    write(
        "POST",
        "/v1/play/{id}/events",
        "playEvents",
        "What happened during play.",
        Some("PlayEvents"),
        "Done",
        PLAY,
    ),
    with_query(
        read(
            "/v1/subtitles",
            "listSubtitles",
            "Files per language, in the order to try.",
            "SubtitleList",
            PLAY,
        ),
        &[
            Query {
                name: "title",
                required: true,
                about: "The title key.",
            },
            Query {
                name: "episode",
                required: false,
                about: "season:episode for a series.",
            },
        ],
    ),
    with_text(
        read(
            "/v1/subtitles/{file}",
            "getSubtitle",
            "Clean cues of one file, or WebVTT shifted by the delay.",
            "SubtitleBody",
            PLAY,
        ),
        "text/vtt",
    ),
    write(
        "POST",
        "/v1/core/step",
        "coreStep",
        "Run one core step for a hosted device.",
        Some("HostedStep"),
        "CoreOut",
        HOSTED,
    ),
    write(
        "POST",
        "/v1/reports",
        "report",
        "A client error report.",
        Some("Report"),
        "Done",
        REPORTS,
    ),
    // Finder, private
    write(
        "POST",
        "/v1/find",
        "find",
        "Look for sources within a budget.",
        Some("FindRequest"),
        "FindAnswer",
        FINDER,
    ),
    read(
        "/v1/find/{job}",
        "findJob",
        "The rest of a partial answer.",
        "FindAnswer",
        FINDER,
    ),
    read(
        "/v1/health",
        "finderHealth",
        "Finder health.",
        "FinderHealth",
        HEALTH,
    ),
];
