// login component
pub const LOGIN: &str = r#"
error = 用户名或密码错误
login_text = 登录
submit = 登录
email = 邮箱地址
password = 密码
to_register_prefix = 还没有账号?
to_register = 去注册
"#;

pub const SEARCH_DOCK: &str = r#"
search = 搜索
cancel = 取消
"#;

pub const USER_INFO: &str = r#"
nickname = 昵称
account = 账号
email = 邮箱
phone = 手机号
address = 地址
birthday = 生日
gender = 性别
male = 男
female = 女
secret = 保密
signature = 个性签名
avatar = 头像
set_avatar = 设置
region = 地区
submit = 保存
cancel = 取消
logout = 退出登录
choose_avatar = 添加头像
"#;

pub const TOP: &str = r#"
msg = 消息
contact = 联系人
"#;

pub const ADD_FRIEND: &str = r#"
no_result = 没有搜索结果
search_prompt = 根据手机号、邮箱、账号搜索
apply = 申请
applied = 已申请
applying = 申请中
apply_msg = 申请消息
remark = 备注:
account = 账号:
nickname = 昵称:
region = 地区:
cancel = 取消
"#;

pub const CALL_COM: &str = r#"
incoming_call = 来电
waiting = 等待中...
connecting = 连接中...
conn_error =  连接错误
stream_error = 没有检测到音视频设备
other_error = 其他错误
unkonw_error = 未知错误
busy = 您正在通话中！
"#;

pub const CONVERSATION: &str = r#"
loading = 正在加载数据...
no_result = 没有搜索结果
image = [图片]
emoji = [表情]
video = [视频]
audio = [语音]
file = [文件]
video_call = [视频通话]
audio_call = [语音通话]
error = [错误]
knock_off_msg = 另一个设备登录了你的账号，如果不是你本人，请检查账号密码。
ok = 确定
"#;

pub const CONTACTS: &str = r#"
new_friends = 新的朋友
no_friends = 没有搜索结果
no_result = 没有搜索结果
"#;

pub const RIGHT_CLICK_PANEL: &str = r#"
delete = 删除
mute = 消息免打扰
un_mute = 取消免打扰
forward = 转发
"#;

pub const MSG_ITEM: &str = r#"
cancel = 已取消
duration = 时间:
busy = 占线
deny = 已拒绝
not_answer = 未接听
"#;

pub const SELECT_FRIENDS: &str = r#"
querying = 正在查询...
error = 查询出错
select_friends = 选择好友
empty_result = 没有搜索结果
submit = 确定
cancel = 取消
"#;

pub const FRIEND_CARD: &str = r#"
apply = 申请
applied = 已申请
apply_msg = 申请消息
remark = 备注:
account = 账号:
nickname = 昵称:
region = 地区:
cancel = 取消
"#;

pub const FRIENDSHIP: &str = r#"
requested = 已申请
go_verify = 前往好友验证
added = 已添加
remark = 备注:
apply_msg = 申请消息:
title = 好友申请验证
message = 验证信息:
accept = 通过申请
cancel = 取消
"#;

pub const SETTING: &str = r#"
setting = 设置
language = 语言:
theme = 主题:
light = 浅色
dark = 暗黑
font_size = 字体大小:
small = 小
medium = 中
large = 大
larger = 更大
"#;

pub const POSTCARD: &str = r#"
account = 账号:
remark = 备注:
region = 地区:
signature = 个性签名:
"#;

pub const ACTION: &str = r#"
# action
send_message = 发消息
voice_call = 语音聊天
video_call = 视频通话
"#;

pub const SENDER: &str = r#"
# sender
send = 发送
file = 文件
submit = 确定
cancel = 取消
no_empty = 发送内容不能为空
input_max_len = 发送内容不能超过
disabled = 暂时无法发送消息
group_dismissed = 群聊已经解散
verify_needed = 对方开启了好友验证，请先通过验证
"#;

pub const RECORDER: &str = r#"
press = 按住讲话
recorde = 录音
stop = 停止
send = 发送
send_mobile = 松开发送
cancel_mobile = 松开取消
cancel = 取消
error = 错误，请检查音频设备
"#;

pub const REGISTER: &str = r#"
avatar = 头像
nickname = 昵称
submit = 注册
email = 邮箱
email_hint = 请输入邮箱地址
pwd_hint = 请输入密码
password = 密码
confirm_pwd = 确认密码
confirm_pwd_hint = 请再次输入密码
code = 验证码
send_code = 发送验证码
re_send_code = 重发
registering = 正在注册...
register_failed = 注册失败，请稍后重试
register_success = 注册成功! 正在跳转登录页面...
to_login_prefix = 已经有账号了?
to_login = 去登陆
"#;

pub const RIGHT_PANEL: &str = r#"
hello = 与挚友开始聊天吧！
"#;

pub const SET_DRAWER: &str = r#"
del_friend = 删除好友
dismiss = 解散群聊
quit = 退出群聊
"#;

pub const SET_WINDOW: &str = r#"
add = 添加
delete = 清空聊天记录
mute = 消息免打扰
remark = 备注
group_desc = 群描述
group_name = 群名称
group_announcement = 群公告
"#;
