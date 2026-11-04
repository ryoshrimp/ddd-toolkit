# Domain macro Serde support / ドメインマクロの Serde 対応

## English

Enable the facade/core `serde` feature and opt in with `#[value_object(serde)]`
or `#[entity(serde)]`. Generated code uses the core's Serde re-export; facade-only
consumers do not need a direct Serde dependency. Direct-core consumers must enable
Serde's derive support as described in [macro compatibility](macro-compatibility.md).

```rust
use ddd_toolkit::ValueObject;

#[derive(ValueObject)]
#[value_object(serde)]
#[serde(rename_all = "snake_case")]
enum GrantType {
    AuthorizationCode,
    ClientCredentials,
    RefreshToken,
}
// AuthorizationCode serializes as "authorization_code" and round-trips.
```

Both macros delegate wire-format rules to Serde derive on a borrowed serialization
helper and an owned deserialization helper. Serialization does not clone fields or
require `Clone` (ValueObject's independently generated traits still have their
usual bounds). After decoding, the configured domain validator runs before a
value is returned. Its errors become `value object validation failed` or
`entity validation failed`; panics still propagate. Entity restoration uses the
supplied ID and generates neither new IDs nor creation events.

### Supported attributes

| Location | Forwarded Serde attributes |
| --- | --- |
| Container | `rename`, `rename_all`, `rename_all_fields`, `deny_unknown_fields`, `tag`, `content`, `untagged`, `transparent`, `bound` |
| Enum variant | `rename`, `rename_all`, `alias`, `skip`, `skip_serializing`, `skip_deserializing`, `other`, `untagged`, `bound` |
| Field | `rename`, `alias`, `default`, `flatten`, `skip`, `skip_serializing`, `skip_deserializing`, `skip_serializing_if`, `serialize_with`, `deserialize_with`, `with`, `borrow`, `bound` |

Serde checks attribute syntax, placement, duplicate options, supported naming
rules, and combinations. Direction-specific `rename`, `rename_all`, and `bound`
are forwarded unchanged. No independent naming-rule implementation is used.
Custom field codecs receive the original field type (serialization borrows it).
Explicit bounds refer to the original generic parameters; borrowing and lifetime
relationships follow Serde derive, including its restriction against a domain
lifetime parameter named `'de`. Function/module paths must resolve from the
helper module (which imports the domain module's names); avoid paths or bounds
that rely on `Self` being the domain type.

Attributes outside this table fail with an explicit macro diagnostic, rather
than being silently ignored. In particular, container `from`, `try_from`, `into`,
`default`, `remote`, `crate`, and variant-level custom codecs are not supported.
Conversions can replace domain construction or introduce additional ownership
requirements; the crate path must remain the selected core re-export. Use a
separate wire DTO and explicit validated conversion for those cases. Field
`default` and `flatten` are supported, but the resulting domain candidate must
still pass validation. All decoded fields, including the ID, are supplied state;
attributes such as `skip`/`default` on an ID intentionally affect that state.

Without customization, single-field ValueObject structs keep their transparent
inner-value format, multi-field named structs keep struct format, tuple structs
keep tuple format, and enums keep externally tagged format. Single-field tuple
variants use Serde's newtype format. Entity structs use struct format even with
only an ID. Helper type names do not replace domain names on the wire. Valid
customizations may intentionally change those formats; a DTO is preferable if
the wire model and domain model need different structures.

Do not combine the macro's Serde opt-in with separately derived `Serialize` /
`Deserialize` on the same domain type. Existing manual/separate Serde
implementations without the opt-in remain possible and are responsible for
validation themselves. Place `serde` last when sharing a ValueObject attribute
with validation: `#[value_object(validate = check, error = Error, serde)]`.

## 日本語

ファサード／コアの `serde` feature を有効にし、`#[value_object(serde)]` または
`#[entity(serde)]` を指定します。生成コードはコアの Serde 再公開を使用するため、
ファサードのみの利用では Serde への直接依存は不要です。コアを直接使う場合の
Serde derive 設定は[互換性ガイド](macro-compatibility.md)を参照してください。

上の例では `AuthorizationCode` が `"authorization_code"` として往復変換されます。
両マクロとも、借用型の直列化ヘルパーと所有型の逆直列化ヘルパーで Serde derive に
形式の解釈を委譲します。直列化自体にフィールドの複製や `Clone` は不要です
（ValueObject の標準トレイトには従来の境界が残ります）。復元後、返却前に既存の
ドメイン検証を実行します。検証エラーは `value object validation failed` または
`entity validation failed` となり、パニックは伝播します。Entity は入力された ID を
維持し、新しい ID や作成イベントを生成しません。

対応属性の一覧は上の表のとおりです。コンテナー、バリアント、フィールドそれぞれに
記載した属性をそのまま Serde に渡し、構文、配置、重複、命名規則、組み合わせの
検査も Serde に委ねます。方向別の `rename`／`rename_all`／`bound` にも対応し、
独自の命名規則は実装しません。フィールド用コーデックは元のフィールド型を扱い、
直列化では借用します。明示的な境界は元のジェネリック引数を参照します。借用と
ライフタイムの制約は Serde derive に従い、ドメイン型のライフタイム名に `'de` は
使用できません。ヘルパーモジュールはドメインモジュールの名前をインポートしますが、
`Self` がドメイン型を指すことに依存した関数パスや境界は避けてください。

表にない属性は黙って無視せず、マクロの診断で拒否します。特にコンテナーの
`from`／`try_from`／`into`／`default`／`remote`／`crate`、バリアント単位の独自
コーデックは非対応です。変換属性はドメイン構築を置き換えたり追加の所有権要件を
生じたりするため、またクレートパスは選択されたコアの再公開に固定するためです。
必要な場合は別の DTO と明示的な検証付き変換を使用してください。フィールドの
`default`／`flatten` は対応しますが、適用後も検証を通過する必要があります。
ID も入力状態の一部なので、ID への `skip`／`default` は意図的にその状態を変えます。

属性がない場合は従来どおり、単一フィールド ValueObject は内包値、複数の名前付き
フィールドは構造体、タプル構造体はタプル、列挙型は外部タグ形式です。単一のタプル
バリアントは Serde の newtype 形式です。Entity は ID のみでも構造体形式です。
ドメイン型名の代わりにヘルパー型名が出力されることはありません。
対応属性による形式の変更は可能ですが、ドメイン構造と通信構造が異なる場合は
別 DTO を推奨します。

同じドメイン型でマクロの Serde 指定と別途の `Serialize`／`Deserialize` derive を
併用しないでください。Serde 指定なしの手書き実装や別途の derive は引き続き可能
ですが、検証は利用者の責任です。ValueObject の検証と同じ属性で指定するときは
`#[value_object(validate = check, error = Error, serde)]` のように `serde` を最後に置きます。
