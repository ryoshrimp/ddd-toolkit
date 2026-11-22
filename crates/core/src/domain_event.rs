/// **English:**<br>
/// Marks a type as an event that describes a meaningful domain occurrence.
///
/// This is a marker trait with no required methods or bounds. Event payloads, serialization,
/// equality, and dispatch behavior are defined by the implementing type and its integrations.
///
/// The trait is useful as a bound rather than for its own methods. For example, it constrains
/// [`AggregateRoot::Event`](crate::AggregateRoot::Event),
/// [`DomainEventPublisher::Event`](crate::DomainEventPublisher::Event),
/// [`AsyncDomainEventPublisher::Event`](crate::AsyncDomainEventPublisher::Event), and
/// [`EventConsumer::Event`](crate::EventConsumer::Event), and it is a supertrait of
/// [`DomainEventMetadata`](crate::DomainEventMetadata),
/// [`DomainEventConversion`](crate::DomainEventConversion), and
/// [`DomainEventDeliveryMetadata`](crate::DomainEventDeliveryMetadata). This list is not
/// exhaustive. Types must opt in explicitly before they can be used where these bounds apply.
/// Using these integrations is optional; the trait adds no behavior of its own.
///
/// **日本語:**<br>
/// 意味のあるドメイン上の出来事を表す型であることを示します。
///
/// 必須メソッドや境界を持たないマーカートレイトです。イベントのペイロード、シリアライズ、
/// 等価性、ディスパッチの動作は実装型と連携先で定義します。
///
/// このトレイトの価値はメソッドではなく境界としての利用にあります。例えば
/// [`AggregateRoot::Event`](crate::AggregateRoot::Event)、
/// [`DomainEventPublisher::Event`](crate::DomainEventPublisher::Event)、
/// [`AsyncDomainEventPublisher::Event`](crate::AsyncDomainEventPublisher::Event)、
/// [`EventConsumer::Event`](crate::EventConsumer::Event) の型を制約し、
/// [`DomainEventMetadata`](crate::DomainEventMetadata)、
/// [`DomainEventConversion`](crate::DomainEventConversion)、
/// [`DomainEventDeliveryMetadata`](crate::DomainEventDeliveryMetadata) のスーパートレイトでもあります。
/// この一覧がすべてではありません。これらの境界では、明示的に実装した型だけを受け付けます。
/// これらの連携の利用は任意で、このトレイト自体は固有の動作を追加しません。
pub trait DomainEvent {}
