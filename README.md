# Pser

`Pser`提供密码保管功能，只需记住一个主密码(main password)，就可以管理所有想要保管的密码。`Pser`也可以用来生成随机密码。

`Pser`使用两个会自动互相同步的密码库文件以避免重装系统或硬盘故障导致的密码库丢失问题，这两个密码库文件只要一个存在，就会自动根据该密码库文件同步创建另一个缺失的密码库文件。

其中一个是主密码库文件，路径我不会告诉你，另一个是从密码库文件，它位于程序所在目录下的`pser.db`文件。

此外，Pser：  
- 支持导入和导出功能  
    - 本程序导出的密码库文件为json格式  
    - 本程序可导入由该程序导出的json密码文件  
    - 本程序可导入由浏览器导出的csv密码文件  
- 支持每分钟限制密码测试次数，可防暴力解密  

## 使用Pser

Pser支持随机密码生成功能和密码保管功能。

如果要使用密码库功能，需先初始化一次密码库，之后无需再初始化。

初始化密码库时，将提示输入一个主密码，这个密码是管理密码库的密码，每次涉及到密码库操作时(如向密码库中新增密码、搜索密码库中的密码等操作)，都需要提供该主密码。如果不想提示输入这个主密码，可设置环境变量`PSER_MAIN_PASSWD`。

```bash
# 初始化密码库
$ pser init

# 可设置环境变量，避免每次都询问主密码
export PSER_MAIN_PASSWD="your_password"
```

### 帮助信息

使用`--help`选项查看帮助信息。

```bash
$ pser --help
管理密码、生成随机密码、导入导出密码

在各个子命令中，任何一个需要提供主密码的地方，都可以通过设置环境变量`PSER_MAIN_PASSWD`来提供， 
如果没有提供，在需要主密码的地方，将交互式提示你输入密码

Usage: pser <COMMAND>

Commands:
  gen     生成随机密码
  init    初始化(创建)密码库
  query   搜索密码库中的密码信息。搜索时，只根据密码的所属url或所属desc进行搜索
  insert  添加或修改已有密码
  rm      删除或清空密码信息
  reset   重置主密码
  import  导入密码信息
  export  导出密码信息为json格式
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### 生成随机密码

使用子命令`gen`生成随机密码。

例如：
```bash
# 生成10个长度为8的随机密码，随机密码中可能包含大小写字母、数字、部分标点字符
$ pser gen -udnp -c 10
Zd6@AYhi
d3@MjcJ(
8LSWx7FT
tNzjt4bM
i2HgivMo
D&ONesO0
N8wL%(Xm
elT5f@HU
y&@iyCvJ
X^sUigq5

# 生成1个长度为12的随机密码，随机密码中可能包含大小写字母、数字，但不包含标点字符
$ pser gen -udn 12
cryO9ejXAI6U
```

详细用法查看帮助信息：

```bash
$ pser gen --help
生成随机密码

注：即便指定了要包含某种字符集，生成的随机密码中可仍然可能会不包含该字符集的字符，
指定包含某种字符集的选项，仅表示生成随机密码时会考虑这类字符集，不代表生成的结果中
一定包含该类字符。可以考虑一次性多个随机密码，从中选择一个或多个。

Usage: pser gen [OPTIONS] [LEN]

Arguments:
  [LEN]
          密码长度，默认长度为8
          [default: 8]

Options:
  -u, --up
          是否要考虑大写字母字符集`[A-Z]`
  -d, --down
          是否要考虑小写字母字符集`[a-z]`
  -n, --num
          是否要考虑数字字符集`[0-9]`
  -p, --punc
          是否要考虑其它ASCII字符集`)(*&^%$#@!~`
  -c, --cnt <CNT>
          生成多个随机密码，每个随机密码一行，默认只生成一个随机密码
          [default: 1]
```

### 修改、重置密码库的主密码

`pser reset`子命令可修改密码库的主密码。

```bash
$ pser reset --new NEW_MAIN_PASSWORD
```

### 添加密码到密码库、修改密码库中的密码

初始化密码库之后，可以管理密码库。其中，`pser insert`子命令可以添加密码和修改密码。

例如：

```bash
# 插入一条密码信息到密码库，下面选项几乎都不是必须提供的
$ pser insert --user peny \
              --url google.com \
              --desc google \
              --email peny@fence.com \
              --phone 12343211234 \
              --passwd 'Pass@word' \
              --comment 'card:622218291928312'

# 查看密码库中已存在的所有密码
$ pser query
-[ UUID: d5963efac5fc461582f78c68e364b20e ]----------------------
账户名(account): peny
所属网站(url): google.com
所属(desc): google
邮箱(email): peny@fence.com
联系方式(phone): 12343211234
密码(passwd): Pass@word
备注(comment): card:622218291928312

# 修改密码库中的密码和邮箱，需提供uuid前缀
$ insert --uuid d5963ef --passwd 'new_password' --email "peny@dugo.com"
```

详细用法参考帮助信息：

```bash
$ pser insert --help
添加或修改已有密码

例如:

添加密码(不指定--uuid选项)：
.    $0 insert --user peny --url google.com --desc google --email peny@qq.com --phone 12343211234 --passwd 'Pass@word' --comment 'card:622218291928312'

修改密码(指定--uuid选项)：只修改密码，其它信息不变
.    $0 insert --uuid asdksld --passwd 'Pass@word'

Usage: pser insert [OPTIONS]

Options:
  -i, --uuid <UUID>
          UUID，指定该选项表示修改已有密码而不是添加密码，
          UUID值可以是前几个字符。如果密码库中找不到指定UUID对应的密码信息，则什么都不做。
  -u, --user <USER>
          账户名/用户名
  -U, --url <URL>
          哪个网站的账户(基于域名)
  -d, --desc <DESC>
          什么账户，可以结合url或者和url二选一 
          例如google的账户，可以记录为`url:google.com`或者`desc:google`，或者两者都记录
  -e, --email <EMAIL>
          邮箱，如果有第二邮箱，可使用备注保存，或者使用逗号分隔多个邮箱，
          如果是修改操作，首字符使用`+`，将表示追加邮箱信息，否则将覆盖已有邮箱信息。
  -P, --phone <PHONE>
          联系方式，如果有第二联系方式，可使用备注保存，或者使用逗号分隔多个联系方式，
          如果是修改操作，首字符使用`+`，将表示追加联系方式，否则将覆盖已有联系方式。
  -p, --passwd <PASSWD>
          密码，可以是单个密码，也可以是逗号分隔的多个单词助记词
  -c, --comment <COMMENT>
          备注，记录额外信息。
          如果是修改操作，首字符使用`+`，将表示追加备注信息，否则将覆盖已有备注信息。
  -h, --help
          Print help (see a summary with '-h')
```

### 搜索、查询密码库

`pser query`子命令可以查询密码库。

```bash
# 不指定任何参数时，列出密码库中已保存的所有密码信息
$ pser query

# 可指定参数，将根据url和desc进行搜索
# 例如，搜索 desc 或 url 中包含 google 字符串的所有密码
$ pser query google
```

### 删除密码库中的密码、清空密码库

`pser rm`子命令用于删除密码信息。

例如

```bash
# 删除 uuid 前缀为 d5963ef 的密码信息
$ pser rm d5963ef

# 删除密码库中的所有密码(使用特殊参数值`all`)，即回到初始化状态
$ pser rm all

# 删除密码库文件(使用特殊参数值`remove`)
$ pser rm remove
```

### 导出密码库

`pser export`子命令可导出密码库中的所有密码，导出格式为json格式。

```bash
$ pser export --output /tmp/pser_passwd.json

$ cat /tmp/pser_passwd.json
{
  "6944dbb9702a4b20922bab99d8dd21c7": {
    "username": "30716506",
    "url": "xui.ptlogin2.com",
    "desc": "xui.ptlogin2.com",
    "email": "",
    "phone": "",
    "passwd": "passwd",
    "comment": "",
    "history": {}
  },
  "d60bcb6c36d74d03935160dd0136d22a": {
    "username": "901566",
    "url": "www.proc.com",
    "desc": "www.proc.com",
    "email": "",
    "phone": "",
    "passwd": "passwd",
    "comment": "",
    "history": {}
  }
}
```

### 导入密码数据

`pser import`可导入密码文件，导入操作是向当前密码库添加被导入的所有新密码。

支持导入两种格式的密码文件：
- 由`pser export`导出的json文件  
- 由浏览器导出的csv文件

```bash
# 导入 json 格式的密码信息文件
$ pser import --src-type json --input /tmp/pser_passwd.json
```

如果要导入的是csv格式的密码文件，要求csv文件的第一行(即csv头部)的至少要包含`username`、`url`和`password`三个字段，通常浏览器导出的密码文件中可能还会包含`name`字段和`note`字段，或其它字段。例如，Edge浏览器导出的csv密码文件的第一行为：
```
name,url,username,password
```

如果csv文件第一行的字段不符合条件，应手动修改为正确的字段名称。并且，note字段通常对应密码库的comment字段，也可手动修改为`comment`。

因此，下面几种都是正确的格式：
```
url,username,password
username,url,password
name,url,username,password
name,url,username,password,comment
```

如果确保了 csv 密码文件格式正确，可导入：
```bash
# 导入 csv 格式的密码信息文件
$ pser import --src-type csv --input /tmp/pser_passwd.csv
```
