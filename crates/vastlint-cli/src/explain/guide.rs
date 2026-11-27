//! Reader-facing explanations for the element and attribute vocabulary.
pub fn element(name: &str) -> Option<&'static str> {
    Some(match name {
        "VAST" => "The response envelope. Its version selects the declared VAST vocabulary. Each Ad below is a separate offer.",
        "Ad" => "One ad offer. A sequence number places it in an ad pod. Its body supplies the creative directly or refers the player to another tag.",
        "InLine" => "Supplies the ad metadata, creative instructions and measurement endpoints in this document.",
        "Wrapper" => "Delegates creative delivery to another VAST response and can add tracking or other instructions to that response.",
        "VASTAdTagURI" | "VASTAdTagURL" => "The address of the next VAST response in the wrapper chain. Its contents must be resolved separately to identify the final creative.",
        "AdSystem" => "Names the system that supplied this ad response.",
        "AdTitle" => "The human-readable title assigned to this ad.",
        "AdCreativeId" => "Supplies an additional identifier for matching the creative across systems.",
        "AdServingId" => "An identifier assigned by the serving system to correlate this ad across the delivery chain.",
        "Advertiser" => "Identifies the advertiser associated with the ad.",
        "Description" => "Free text describing the ad or its campaign.",
        "Category" | "BlockedAdCategories" => "Declares a category or category restriction in the classification system named by authority.",
        "Pricing" => "A declared price and currency under the named pricing model. Delivery alone does not establish a charge.",
        "Survey" => "A survey resource associated with the ad.",
        "Expires" => "Advises how long, in seconds, a recipient may wait after receiving this response before using the ad.",
        "Error" => "A reporting URL for an ad-serving or playback failure. The player can substitute an error code when it reports the failure.",
        "Impression" => "A beacon URL the player is instructed to request when it records the ad impression. Its presence alone does not establish delivery.",
        "ViewableImpression" => "Groups measurement endpoints for the possible viewability outcomes.",
        "Viewable" => "A beacon for an impression measured as viewable.",
        "NotViewable" => "A beacon for an impression measured below the applicable viewability criteria.",
        "ViewUndetermined" => "A beacon for an impression whose viewability could not be determined.",
        "Creatives" => "Groups the creative instructions belonging to this ad.",
        "Creative" => "Groups a creative format with its identifiers and optional extensions. One ad may provide several creatives.",
        "UniversalAdId" => "Identifies the creative within the registry named by idRegistry. It can be compared across serving systems.",
        "Linear" => "A creative presented in the content timeline. The player selects a compatible rendition and uses the declared duration and tracking.",
        "NonLinearAds" => "Groups non-linear presentations with their tracking. They can occupy a surface over, alongside, or during a pause in content.",
        "NonLinear" => "One non-linear presentation with declared dimensions and resource choices. Its placement depends on the format signals and the receiving player.",
        "CompanionAds" => "Groups supplemental creatives that can accompany the primary ad. The required attribute can constrain how many should be displayed.",
        "Companion" => "A supplemental creative intended for a companion slot. Its dimensions and resource type help a player select a compatible slot.",
        "Duration" => "Declared running time in hours, minutes and seconds. Players can use it to schedule progress events.",
        "MediaFiles" => "Groups media renditions and related delivery resources. Multiple renditions can represent alternatives for the same creative.",
        "MediaFile" => "A media rendition with a declared delivery method and MIME type. A player selects a supported encoding and size.",
        "Mezzanine" => "A high-quality source asset for transcoding or server-side processing.",
        "InteractiveCreativeFile" => "An executable interactive resource. Its framework and MIME type tell a compatible player how to host it.",
        "ClosedCaptionFiles" => "Groups available caption tracks for the creative.",
        "ClosedCaptionFile" => "A separate caption track, identified by language and file type when supplied.",
        "AdParameters" => "Creative-specific initialization data passed to the ad or its interactive runtime.",
        "StaticResource" => "A file-backed creative resource. creativeType identifies its declared media type, such as image/png.",
        "HTMLResource" => "HTML markup supplied for rendering inside a compatible creative surface. The report displays it as text.",
        "IFrameResource" => "The URL of a document intended to render inside a frame.",
        "TrackingEvents" => "Groups event beacons for the surrounding creative or verification resource.",
        "Tracking" => "A reporting URL associated with the event attribute. Multiple URLs for an event can belong to different measurement systems.",
        "VideoClicks" => "Groups navigation and click measurement for a linear creative.",
        "ClickThrough" | "NonLinearClickThrough" | "CompanionClickThrough" | "IconClickThrough" | "OverviewClickThrough" => "The destination to open when the viewer activates this creative surface.",
        "ClickTracking" | "NonLinearClickTracking" | "CompanionClickTracking" | "IconClickTracking" => "A measurement endpoint for a click on this surface. The navigation destination is declared separately.",
        "CustomClick" | "NonLinearCustomClick" => "A URL associated with a custom click interaction defined by the creative.",
        "Icons" => "Groups program icons that may be presented with the creative.",
        "Icon" => "A program or disclosure icon with its own display resource, geometry, and click handling.",
        "IconClicks" => "Groups the navigation and measurement instructions for this icon.",
        "IconViewTracking" => "A beacon for display of the icon itself.",
        "IconClickFallbackImages" => "Groups fallback images for devices that cannot open the icon click-through.",
        "IconClickFallbackImage" => "A fallback presentation associated with an icon click.",
        "AltText" => "Text describing a visual resource for accessibility or fallback display.",
        "CompanionAltText" => "An alternative text description for the companion creative.",
        "AdVerifications" => "Groups independent verification resources associated with the ad.",
        "Verification" => "Identifies a verification vendor and its resources, initialization parameters, and reporting instructions.",
        "JavaScriptResource" => "A script intended for a compatible verification environment, such as OMID.",
        "ExecutableResource" => "A verification program for the runtime identified in its attributes.",
        "VerificationParameters" => "Vendor-specific initialization data passed to the verification resource.",
        "Extensions" | "CreativeExtensions" => "Groups additional metadata scoped to its enclosing ad or creative.",
        "Extension" | "CreativeExtension" => "A named extension payload. Its type and implementing system determine how descendants are interpreted.",
        "CreativeId" => "Binds a CTV portfolio extension to the creative with this identifier.",
        "plcmt" => "AdCOM placement subtype identifying the intended experience.",
        "pos" => "AdCOM placement position describing where the ad is intended to appear.",
        "playbackmethod" => "AdCOM playback method describing the trigger and initial sound behavior.",
        "attr" => "An AdCOM creative attribute declaring a property of the delivered experience.",
        "QrCodeImageUrl" => "Image resource containing the QR code.",
        "QrCodeScanUrl" => "Destination encoded by the QR code for a viewer to scan.",
        "QrCodePosition" => "Declared location for placing the QR code in the creative.",
        "QrCodeSize" => "Declared display size of the QR code.",
        "AdChoices" => "Disclosure information associated with the ad choices program.",
        "FlashResource" => "A legacy Flash creative resource.",
        _ => return None,
    })
}

pub fn attribute(name: &str) -> Option<&'static str> {
    Some(match name {
        "id" => "Identifier scoped to this element.",
        "adId" | "AdID" | "idValue" => "Creative or ad identifier supplied by the sender.",
        "version" => "Version declared by this element.",
        "sequence" => "Order within the enclosing ad pod or creative sequence.",
        "type" | "creativeType" => "Declared resource type or extension discriminator.",
        "apiFramework" => "Runtime or API the resource expects.",
        "event" => "Condition associated with this beacon.",
        "width" | "height" => "Declared display or asset dimension for this element.",
        "expandedWidth" | "expandedHeight" => "Dimension of the expanded presentation.",
        "assetWidth" | "assetHeight" => "Source asset dimension.",
        "delivery" => {
            "How the player obtains the media, such as progressive download or streaming."
        }
        "bitrate" | "minBitrate" | "maxBitrate" => "Declared bitrate in kilobits per second.",
        "codec" => "Codec the receiving player must support for this rendition.",
        "offset" => "Time or percentage offset associated with this event or resource.",
        "skipoffset" => "Elapsed time or percentage at which skipping becomes available.",
        "scalable" => "Whether the asset may be resized.",
        "maintainAspectRatio" => {
            "Whether the original proportions should be preserved when resizing."
        }
        "duration" | "minSuggestedDuration" => {
            "Declared duration or suggested minimum exposure time."
        }
        "variableDuration" => "Whether the interactive experience can change the running time.",
        "idRegistry" => "Registry that defines the identifier.",
        "authority" => "System defining the category vocabulary.",
        "ext" => "Vocabulary identifying the extension, for example adcom.",
        "vendor" => "Verification provider identifier.",
        "browserOptional" => {
            "Whether the verification resource can operate without a browser environment."
        }
        "language" | "lang" => "Language of the supplied resource.",
        "program" => "Program represented by this icon.",
        "xPosition" | "yPosition" => "Coordinate or alignment within the display surface.",
        "fileSize" => "Declared size in bytes.",
        "pxratio" => "Pixel ratio for rendering the resource.",
        "currency" => "Currency of the declared price.",
        "model" => "Pricing model applied to this value.",
        "required" => "Companion display requirement for this group.",
        "adSlotID" | "adSlotId" => "Slot identifier used for matching a placement.",
        "followAdditionalWrappers" => "Whether additional wrappers may be followed.",
        "allowMultipleAds" => "Whether the downstream response may supply multiple ads.",
        "fallbackOnNoAd" | "fallback" => "Fallback behavior declared for this resource or wrapper.",
        "adType" => "Declared ad class, such as video or audio.",
        "mediaType" => "Presentation type of the media, such as 2D or 360.",
        "conditionalAd" => "Whether serving this ad depends on a condition.",
        "xmlEncoded" => "Whether the supplied parameters contain XML-encoded data.",
        "renderingMode" => "When the companion should be rendered relative to the main ad.",
        "size" => "Declared size of the resource.",
        _ => return None,
    })
}

pub fn event(name: &str) -> &str {
    match name {
        "creativeView" => "The creative is displayed.",
        "loaded" => "The creative is loaded and ready.",
        "start" => "Creative playback begins.",
        "firstQuartile" => "Playback reaches 25% of the duration.",
        "midpoint" => "Playback reaches 50% of the duration.",
        "thirdQuartile" => "Playback reaches 75% of the duration.",
        "complete" => "Playback reaches the end.",
        "progress" => "Playback reaches the declared offset.",
        "pause" => "Playback is paused.",
        "resume" => "Playback resumes after a pause.",
        "skip" => "The viewer skips the ad.",
        "mute" => "Audio is muted.",
        "unmute" => "Audio is restored.",
        "rewind" => "Playback moves backward.",
        "fullscreen" => "The player enters fullscreen.",
        "exitFullscreen" => "The player leaves fullscreen.",
        "expand" | "playerExpand" => "The creative or player expands.",
        "collapse" | "playerCollapse" => "The creative or player returns to its smaller size.",
        "close" | "closeLinear" => "The viewer closes the creative.",
        "acceptInvitation" | "acceptInvitationLinear" => {
            "The viewer accepts the creative's invitation to interact."
        }
        "verificationNotExecuted" => "The player reports that verification did not execute.",
        "overlayViewDuration" => "Reports the exposure duration of an overlay.",
        "notUsed" => "The ad was available but was not used.",
        "otherAdInteraction" => "Another interaction with the ad occurred.",
        _ => "The receiver determines support for this event name.",
    }
}

pub fn signal(name: &str, value: &str) -> Option<&'static str> {
    let values: &[&str] = match name {
        "plcmt" => &[
            "",
            "Instream",
            "Accompanying content",
            "Interstitial",
            "Standalone",
            "Pause",
            "Screensaver",
            "Overlay",
            "Squeezeback",
            "In-scene",
        ],
        "pos" => &[
            "Unspecified",
            "Above the fold",
            "Locked",
            "Below the fold",
            "Header",
            "Footer",
            "Sidebar",
            "Fullscreen",
            "Partial screen",
            "Top left",
            "Top right",
            "Surround",
            "Double box",
            "Double box with background",
            "Bottom left",
            "Bottom right",
            "L-shape",
            "Reverse L-shape",
        ],
        "playbackmethod" => &[
            "",
            "Autoplay with sound",
            "Autoplay muted",
            "Click to play",
            "Mouse over",
            "Viewport entry with sound",
            "Viewport entry muted",
            "Continuous playback",
            "Pause with sound",
            "Pause muted",
            "Idle with sound",
            "Idle muted",
        ],
        "attr" => {
            return match value {
                "19" => Some("Advertiser QR code"),
                "20" => Some("Alpha transparency"),
                "21" => Some("Static visual"),
                "22" => Some("Limited motion"),
                "23" => Some("Full motion"),
                _ => None,
            }
        }
        _ => return None,
    };
    values
        .get(value.parse::<usize>().ok()?)
        .copied()
        .filter(|s| !s.is_empty())
}
