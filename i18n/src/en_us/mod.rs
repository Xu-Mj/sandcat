// login component
pub const LOGIN: &str = r#"
error = account or password is incorrect
login_text = SignIn
submit = login
email = e-mail
password = password
to_register_prefix = don't have an account?
to_register = REGISTER NOW
"#;

pub const TOP: &str = r#"
msg = Messages
contact = Contacts
"#;

pub const SEARCH_DOCK: &str = r#"
search = Search
cancel = Cancel
"#;

pub const USER_INFO: &str = r#"
nickname = nicname
account = account
change_pwd = set_pwd
email = email
phone = phone
address = address
birthday = birthday
gender = gender
male = male
female = female
secret = secret
signature = signature
avatar = avatar
set_avatar = SET
region = region
submit = Submit
cancel = Cancel
logout = Logout
choose_avatar = Choose
"#;

pub const CHANGE_PWD: &str = r#"
new_pwd = new pwd
pwd_hint = new pwd
confirm_pwd = repeat
confirm_pwd_hint = confirm password
code = code
re_send_code = resend code
send_code = send code
submit = Submit
cancel = Cancel
"#;

pub const CALL_COM: &str = r#"
incoming_call = Call you
waiting = Waiting...
connecting = Connecting...
conn_error = Connection error
stream_error = Cannot detect related device
other_error = Other error
unkonw_error = Unkonw error
busy = Busy
"#;

pub const CONVERSATION: &str = r#"
loading = Loading...
no_result = No Result
# content type
image = [Image]
emoji = [Emoji]
video = [Video]
audio = [Audio]
file = [File]
text = [Text]
video_call = [Video Call]
audio_call = [Voice Call]
error = [ERROR]
knock_off_msg = Another device has logged in your account, if it is not you, please check your account password.
ok = OK
"#;

pub const CONTACTS: &str = r#"
new_friends = New Friends
no_friends = No Friends
no_groups = No Groups
no_result = No Result
"#;

pub const MSG_ITEM: &str = r#"
cancel = Canceled
duration = Duration:
deny = Denied
busy = Busy
not_answer = Not answered
"#;

pub const ADD_FRIEND: &str = r#"
no_result = No Result
search_prompt = search by phone number or email address or account
apply = Apply
applied = applied
applying = applying
apply_msg = apply message
remark = remark
cancel = Cancel
nickname = nickname:
account = account:
region = region:
"#;

pub const RIGHT_CLICK_PANEL: &str = r#"
# right click panel
delete = Delete
mute = Mute
un_mute = Un-mute
pin = Pin
un_pin = Un-pin
forward = Forward
related = Quote
"#;

pub const SELECT_FRIENDS: &str = r#"
querying = Querying
error = Query Error
select_friends = Select Friends
empty_result = No Result
submit = Submit
cancel = Cancel
"#;

pub const FRIEND_CARD: &str = r#"
add = Add Friend
apply = Apply
applied = applied
apply_msg = apply message
remark = remark
cancel = Cancel
nickname = nickname:
account = account:
region = region:
"#;

pub const FRIENDSHIP: &str = r#"
# friendship
requested = Requested
go_verify = GoVerify
added = Added
remark = remark:
apply_msg = Apply Message:
title = Friendship Verification
message = Message:
accept = Accept
cancel = Cancel
"#;

pub const SETTING: &str = r#"
setting = Setting
language = Language:
theme = Theme:
transparent = transparent:
light = Light
dark = Dark
font_size = Font Size:
small = Small
medium = Medium
large = Large
larger = Larger
"#;

pub const POSTCARD: &str = r#"
account = account:
remark = remark
region = region:
signature = signature
announcement = anno
"#;

pub const ACTION: &str = r#"
# action
send_message = Message
voice_call = VoiceCall
video_call = VideoCall
"#;

pub const SENDER: &str = r#"
send = Send
#file = File
submit = Submit
cancel = Cancel
no_empty = CAN NOT SEND AN EMPTY MESSAGE
input_max_len = input len more than
disabled = CAN NOT SEND FOR NOW
group_dismissed = THE GROUP HAS BEEN DISMISSED
verify_needed = FRIEND ENABLED THE FRIEND VERIFICATION, PLEASE PASS IT FIRST
image = [Image]
emoji = [Emoji]
video = [Video]
audio = [Audio]
file = [File]
text = [Text]
video_call = [Video Call]
audio_call = [Voice Call]
error = [ERROR]
"#;

pub const RECORDER: &str = r#"
press = press to speak
record = record
stop = stop
send = send
send_mobile = Release to send
cancel = cancel
cancel_mobile = Release to cancel
error = ERROR: check your recorder
"#;

pub const REGISTER: &str = r#"
submit = register
avatar = avatar
nickname = nickname
email_hint = enter your e-mail
email = e-mail
pwd_hint = enter your password
password = password
confirm_pwd = confirm
confirm_pwd_hint = input password again
code = code
send_code = send code
re_send_code = re-send
registering = registering...
register_failed = register failed
register_success = register success! redirecting...
to_login_prefix = already have an account?
to_login = LOGIN NOW
"#;

pub const RIGHT_PANEL: &str = r#"
querying = Querying...
hello = Let's Chat
"#;

pub const SET_DRAWER: &str = r#"
del_friend = Delete
dismiss = Dismiss
quit = Quit
"#;

pub const SET_WINDOW: &str = r#"
add = Add
remove = Remove
delete = CleanChatHistory
mute = Mute
remark = Remark
group_desc = GroupDesc
group_name = GroupName
group_announcement = GroupAnno
"#;

// 改成英文
pub const NOTIFICATION: &str = r#"
Internal = Internal Error
UnknownError = Unknown Error
Network = Network Error
LocalNotFound = Local Resource Not Found
NotFound = Resource Not Found
ServerError = Server Error
UnAuthorized = Unauthorized
BadRequest = Bad Request
AccountOrPassword = Account or Password Error
CodeIsExpired = Code Is Expired
CodeIsInvalid = Code Is Invalid
MsgSendError= Msg Send Error
WsConnError = WebSocket Connection Error
WsClosed = WebSocket Closed
"#;
