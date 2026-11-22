/// **English:**<br>
/// Constructs a domain product from owned input, reporting a typed construction error.
/// Validation, identifier allocation, and side effects are chosen by the implementation;
/// this trait does not require the product to be an aggregate or provide persistence.
/// A shared receiver does not require `Send`, `Sync`, or side-effect-free construction.
///
/// Benefits: code can be generic over `F: DomainFactory<Input>` and depend on the construction role
/// instead of a concrete type, tests can pass a stub factory, and the trait names the construction
/// step in signatures and docs. Using it is optional: if construction happens in one place and is
/// never swapped, a plain function or associated constructor is enough.
///
/// **日本語:**<br>
/// 所有する入力からドメインの生成物を構築し、型付きの構築エラーを返します。
/// 検証、識別子の割り当て、副作用は実装が定めます。生成物を集約に制限せず、永続化も提供しません。
/// 共有参照のレシーバーは、`Send`、`Sync`、副作用のない構築を要求するものではありません。
///
/// 利点: `F: DomainFactory<Input>` のようにジェネリックな境界として使えば、具体型ではなく
/// 構築の役割に依存でき、テストではスタブのファクトリを渡せます。また、シグネチャや
/// ドキュメントで構築処理を型名で明示できます。利用は任意で、1 か所でしか構築せず
/// 差し替えないなら、通常の関数や関連関数で十分です。
pub trait DomainFactory<Input> {
    /// **English:**<br>
    /// Value returned after successful construction; no domain trait bound is imposed.
    ///
    /// **日本語:**<br>
    /// 構築成功時に返す値です。ドメイントレイトの境界は課しません。
    type Product;

    /// **English:**<br>
    /// Implementation-defined reason that construction could not return a product.
    ///
    /// **日本語:**<br>
    /// 生成物を返せなかった理由を表す実装定義のエラーです。
    type Error;

    /// **English:**<br>
    /// Consumes `input` and returns a product or a construction error.
    /// The implementation specifies validation rules, failure side effects, and any panics.
    ///
    /// **日本語:**<br>
    /// `input` を消費し、生成物または構築エラーを返します。
    /// 検証規則、失敗時の副作用、パニックの条件は実装が定めます。
    fn create(&self, input: Input) -> Result<Self::Product, Self::Error>;
}
