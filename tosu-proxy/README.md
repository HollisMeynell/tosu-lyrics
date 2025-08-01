# 后端接口 文档

消息通过 WebSocket, 使用 json 格式进行传输

目前消息分为两种, 一个是歌词事件, 一个是配置事件

其中歌词事件仅后端发送

配置事件包含 **提交**, **响应**, **广播** 三种类型

> [!IMPORTANT]
> 所有 json 的 `key` 均使用小驼峰命名法
>

## 歌词事件

类型:

### LyricLine

| name   | type   | description | required |
|:-------|:-------|:------------|:--------:|
| first  | string | 主要歌词        |    N     |
| second | string | 次要歌词        |    N     |

### Lyric

| name     | type      | description | required |
|:---------|:----------|:------------|:--------:|
| type     | string    | "lyric"     |    Y     |
| previous | LyricLine | 上一条         |    N     |
| current  | LyricLine | 当前条         |    N     |
| next     | LyricLine | 下一条         |    N     |
| nextTime | number    | 持续时间(ms)    |    Y     |
| sequence | enum      | "up"/"down" |    Y     |

示例:

```json
{
  "type": "lyric",
  "previous": {
    "first": "还记得 你说家是唯一的城堡",
    "second": ""
  },
  "current": {
    "first": "随着稻香河流继续奔跑",
    "second": ""
  },
  "next": {
    "first": "微微笑 小时候的梦我知道",
    "second": ""
  },
  "nextTime": 1000,
  "sequence": "up"
}
```

## 设置事件

> 约定字段 `key` 使用小驼峰命名
>
> `key` 命名时, 提交设置与广播通常使用 `set` 开头, 查询使用 `get` 开头, 响应使用 `rep` 开头
>
> 提交设置与设置广播通常不会出现 `error` 以及 `echo`

类型:

### 封装类

| name  | type   | description                    | required |
|:------|:-------|:-------------------------------|:--------:|
| type  | string | "setting"                      |    Y     |
| key   | string | 设置类型, 与`value`的类型绑定            |    Y     |
| value | ?      | 具体数据类型, 与 `key` 绑定, 可以为 `null` |    N     |
| error | string | 错误信息, 通常与 `value` 互斥           |    N     |
| echo  | string | 如果请求中包含, 响应同样包含相同的值            |    N     |

示例:

```json
{
  "type": "setting",
  "key": "getLyricList",
  "value": {
    "title": "稻香",
    "artist": "周杰伦",
    "length": 203000
  },
  "echo": "3c50"
}
```

```json
{
  "type": "setting",
  "key": "repLyricList",
  "value": {
    "QQ": [
      {
        "title": "稻香",
        "artist": "周杰伦",
        "length": 203000,
        "key": "003aAYrm3GE0Ac"
      }
    ]
  },
  "echo": "3c50"
}
```

```json
{
  "type": "setting",
  "key": "repLyricList",
  "error": "Search result is empty",
  "echo": "3c50"
}
```

## 设置事件 子列表

| key                | type                                | description | done |
|:-------------------|-------------------------------------|:------------|:----:|
| setClear           | null                                | 清空当前显示的歌词   |  N   |
| setFont            | [BaseLyricSetter](#BaseLyricSetter) | 字体          |  N   |
| setFontSize        | [BaseLyricSetter](#BaseLyricSetter) | 字体大小        |  N   |
| setAlignment       | [BaseLyricSetter](#BaseLyricSetter) | 对齐方式        |  N   |
| setColor           | [BaseLyricSetter](#BaseLyricSetter) | 字体颜色        |  N   |
| setTranslationMain | bool                                | 翻译为主歌词      |  N   |
| setSecondShow      | bool                                | 显示副歌词       |  N   |
| setLyricSource     | string                              | 切换指定`key`歌词 |  N   |
| getLyricList       | [SongInfoList](#SongInfoList)       | 获取搜索结果      |  N   |
| getAllLyric        | [LyricLine[]](#LyricLine)           | 获取当前曲子完整歌词  |  N   |
| setBlock           | [BlockItem](#BlockItem)             | 获取当前曲子完整歌词  |  N   |
| getBlockList       | [BlockItem[]](#BlockItem)           | 获取当前曲子完整歌词  |  N   |
| setUnblock         | [BlockItem](#BlockItem)             | 获取当前曲子完整歌词  |  N   |
| getClearCount      | number                              | 已缓存歌词的数量    |  N   |
| setClearCache      | null                                | 清空缓存        |  N   |
| getLyricOffset     | number                              | 查看当前歌词的偏移   |  N   |
| setLyricOffset     | number                              | 修改当前歌词的偏移   |  N   |

### BaseLyricSetter

基础配置的结构, 通常包含 主/副 歌词的单个配置

| name   | type   | description | required |
|:-------|:-------|:------------|:--------:|
| first  | string | 主要歌词的配置     |    N     |
| second | string | 次要歌词的配置     |    N     |

### SongInfo

歌曲信息

| name   | type   | description | required |
|:-------|:-------|:------------|:--------:|
| title  | string | 曲名          |    Y     |
| artist | string | 作者          |    Y     |
| length | string | 时常(ms)      |    Y     |
| key    | string | 歌曲ID        |    Y     |

### SongInfoList

歌曲信息列表

| name    | type                    | description | required |
|:--------|:------------------------|:------------|:--------:|
| QQ      | [SongInfo[]](#SongInfo) | qq 歌词源      |    Y     |
| Netease | [SongInfo[]](#SongInfo) | 网易源         |    Y     |

### BlockItem

黑名单

| name  | type   | description     | required |
|:------|:-------|:----------------|:--------:|
| bid   | number | 三选一             |    N     |
| sid   | number | 三选一             |    N     |
| title | string | 三选一(上传曲名正则表达式?) |    N     |

## 其他 HTTP 接口 (画大饼):

### GET - 查询歌曲时常

查询当前歌曲的时间长度 (毫秒)

- `/audio/len`

参数:

- `path`: 文件路径

### POST - 上传字体

上传字体文件

- `/font/upload`

使用 form 上传, 取第一个文件

### GET - 下载字体

- `/font/download`

下载上次上传的文件