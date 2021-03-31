#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubAckReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 1 message proceeds
    Success = 0x00,
    /// 16[0x10], The message is accepted but there are no subscribers. This is sent only by the Server.
    /// If the Server knows that there are no matching subscribers, it MAY use this Reason Code instead of 0x00 (Success)
    NoMatchingSubscribers = 0x10,
    /// 128[0x80], The receiver does not accept the publish but either does not want to reveal the reason,
    /// or it does not match one of the other values
    UnspecifiedError = 0x80,
    /// 131[0x83], The `PUBLISH` is valid but the receiver is not willing to accept it.
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The `PUBLISH` is not authorized
    NotAuthorized = 0x87,
    /// 144[0x90], The Topic Name is not malformed, but is not accepted by this Client or Server
    TopicNameInvalid = 0x90,
    /// 145[0x91], The Packet Identifier is already in use. This might indicate a mismatch in the Session State between
    /// the Client and Server
    PacketIdentifierInUse = 0x91,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded
    QuotaExceeded = 0x97,
    /// 153[0x99], The payload format does not match the specified Payload Format Indicator
    PayloadFormatInvalid = 0x99,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubRecReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 2 message proceeds
    Success = 0x00,
    /// 16[0x10], The message is accepted but there are no subscribers. This is sent only by the Server.
    /// If the Server knows that there are no matching subscribers, it MAY use this Reason Code instead of 0x00 (Success)
    NoMatchingSubscribers = 0x10,
    /// 128[0x80], The receiver does not accept the publish but either does not want to reveal the reason,
    /// or it does not match one of the other values
    UnspecifiedError = 0x80,
    /// 131[0x83], The PUBLISH is valid but the receiver is not willing to accept it
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The PUBLISH is not authorized
    NotAuthorized = 0x87,
    /// 144[0x90], The Topic Name is not malformed, but is not accepted by this Client or Server
    TopicNameInvalid = 0x90,
    /// 145[0x91], The Packet Identifier is already in use. This might indicate a mismatch in the
    /// Session State between the Client and Server.
    PacketIdentifierInUse = 0x91,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded
    QuotaExceeded = 0x97,
    /// 153[0x99], The payload format does not match the one specified in the Payload Format Indicator
    PayloadFormatInvalid = 0x99,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubRelReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 2 message proceeds
    Success = 0x00,
    /// 146[0x92], The Packet Identifier is not known. This is not an error during recovery,
    /// but at other times indicates a mismatch between the Session State on the Client and Server.
    PacketIdentifierNotFound = 0x92,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubCompReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 2 message proceeds
    Success = 0x00,
    /// 146[0x92], The Packet Identifier is not known. This is not an error during recovery,
    /// but at other times indicates a mismatch between the Session State on the Client and Server.
    PacketIdentifierNotFound = 0x92,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubscribeReasonCode {
    /// 0[0x00], The subscription is accepted and the maximum QoS sent will be QoS 0.
    /// This might be a lower QoS than was requested.
    GrantedQos0 = 0x00,
    /// 1[0x01], The subscription is accepted and the maximum QoS sent will be QoS 1.
    /// This might be a lower QoS than was requested.
    GrantedQos1 = 0x01,
    /// 2[0x02], The subscription is accepted and any received QoS will be sent to this subscription.
    GrantedQos2 = 0x02,
    /// 128[0x80], The subscription is not accepted and the Server either does not wish to reveal the
    /// reason or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// 131[0x83], The SUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The Client is not authorized to make this subscription.
    NotAuthorized = 0x87,
    /// 143[0x8F], The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInvalid = 0x8F,
    /// 145[0x91], The specified Packet Identifier is already in use.
    PacketIdentifierInUse = 0x91,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded = 0x97,
    /// 158[0x9E], The Server does not support Shared Subscriptions for this Client.
    SharedSubscriptionNotSupported = 0x9E,
    /// 161[0xA1], The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifierNotSupported = 0xA1,
    /// 162[0xA2], The Server does not support Wildcard subscription; the subscription is not accepted.
    WildcardSubscriptionNotSupported = 0xA2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnSubscribeReasonCode {
    /// 0[0x00], The subscription is deleted.
    Success = 0x00,
    /// 17[0x11], No matching Topic Filter is being used by the Client.
    NoSubscriptionFound = 0x11,
    /// 128[0x80], The unsubscribe could not be completed and the Server
    /// either does not wish to reveal the reason or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// 131[0x83], The UNSUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The Client is not authorized to unsubscribe.
    NotAuthorized = 0x87,
    /// 143[0x8F], The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInValid = 0x8F,
    /// 145[0x91], The specified Packet Identifier is already in use.
    PacketIdentifierInUse = 0x91,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisconnectReasonCode {
    /// 0[0x00], Close the connection normally. Do not send the Will Message.
    NormalDisconnection = 0x00,
    /// 4[0x04], The Client wishes to disconnect but requires that the Server also publishes its Will Message.
    DisconnectWithWillMessage = 0x04,
    /// 128[0x80], The Connection is closed but the sender either does not wish to reveal the reason, or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// 129[0x81], The received packet does not conform to this specification.
    MalformedPacket = 0x81,
    /// 130[0x82], An unexpected or out of order packet was received.
    ProtocolError = 0x82,
    /// 131[0x83], The packet received is valid but cannot be processed by this implementation.
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The request is not authorized.
    NotAuthorized = 0x87,
    /// 137[0x89], The Server is busy and cannot continue processing requests from this Client.
    ServerBusy = 0x89,
    /// 139[0x8B], The Server is shutting down.
    ServerShuttingDown = 0x8B,
    /// 141[0x8D], The Connection is closed because no packet has been received for 1.5 times the Keepalive time.
    KeepAliveTimeout = 0x8D,
    /// 142[0x8E], Another Connection using the same ClientID has connected causing this Connection to be closed.
    SessionTakenOver = 0x8E,
    /// 143[0x8F], The Topic Filter is correctly formed, but is not accepted by this Sever.
    TopicFilterInvalid = 0x8F,
    /// 144[0x90], The Topic Name is correctly formed, but is not accepted by this Client or Server.
    TopicNameInvalid = 0x90,
    /// 147[0x93], The Client or Server has received more than Receive Maximum publication for which it has not sent PUBACK or PUBCOMP.
    ReceiveMaximumExceeded = 0x93,
    /// 148[0x94], The Client or Server has received a PUBLISH packet containing a Topic Alias which is greater than the Maximum Topic Alias it sent in the CONNECT or CONNACK packet.
    TopicAliasInvalid = 0x94,
    /// 149[0x95], The packet size is greater than Maximum Packet Size for this Client or Server.
    PacketTooLarge = 0x95,
    /// 150[0x96], The received data rate is too high.
    MessageRateTooHigh = 0x96,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded = 0x97,
    /// 152[0x98], The Connection is closed due to an administrative action.
    AdministrativeAction = 0x98,
    /// 153[0x99], The payload format does not match the one specified by the Payload Format Indicator.
    PayloadFormatInvalid = 0x99,
    /// 154[0x9A], The Server has does not support retained messages.
    RetainNotSupported = 0x9A,
    /// 155[0x9B], The Client specified a QoS greater than the QoS specified in a Maximum QoS in the CONNACK.
    QosNotSupported = 0x9B,
    /// 156[0x9C], The Client should temporarily change its Server.
    UseAnotherServer = 0x9C,
    /// 157[0x9D], The Server is moved and the Client should permanently change its server location.
    ServerMoved = 0x9D,
    /// 158[0x9E], The Server does not support Shared Subscriptions.
    SharedSubscriptionNotSupported = 0x9E,
    /// 159[0x9F], This connection is closed because the connection rate is too high.
    ConnectionRateExceeded = 0x9F,
    /// 160[0xA0], The maximum connection time authorized for this connection has been exceeded.
    MaximumConnectTime = 0xA0,
    /// 161[0xA1], The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifiersNotSupported = 0xA1,
    /// 162[0xA2], The Server does not support Wildcard subscription; the subscription is not accepted.
    WildcardSubscriptionsNotSupported = 0xA2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthenticateReasonCode {
    /// 0[0x00], Authentication is successful
    Success = 0x00,
    /// 24[0x18], Continue the authentication with another step
    ContinueAuthentication = 0x18,
    /// 25[0x19], Initiate a re-authentication
    ReAuthenticate = 0x19,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectReasonCode {
    /// 0[0x00], Connection accepted
    Success = 0x00,
    /// 128[0x80], The Server does not wish to reveal the reason for the failure,
    /// or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// 129[0x81], Data within the `CONNECT` packet could not be correctly parsed
    MalformedPacket = 0x81,
    /// 130[0x82], Data in the `CONNECT` packet does not conform to this specification
    ProtocolError = 0x82,
    /// 131[0x83], The `CONNECT` is valid but is not accepted by this Server
    ImplementationSpecificError = 0x83,
    /// 132[0x84], The Server does not support the version of the MQTT protocol requested by the Client.
    UnsupportedProtocolVersion = 0x84,
    /// 133[0x85], The Client Identifier is a valid string but is not allowed by the Server
    ClientIdentifierNotValid = 0x85,
    /// 134[0x86], The Server does not accept the User Name or Password specified by the Client
    BadUsernameOrPassword = 0x86,
    /// 135[0x87], The Client is not authorized to connect
    NotAuthorized = 0x87,
    /// 136[0x88], The MQTT Server is not available
    ServerUnavailable = 0x88,
    /// 137[0x89], The Server is busy. Try again later
    ServerBusy = 0x89,
    /// 138[0x8A], This Client has been banned by administrative action. Contact the server administrator
    Banned = 0x8A,
    /// 140[0x8C], The authentication method is not supported or does not match the authentication method currently in use
    BadAuthenticationMethod = 0x8C,
    /// 144[0x90], The Will Topic Name is not malformed, but is not accepted by this Server
    TopicNameInvalid = 0x90,
    /// 149[0x95], The `CONNECT` packet exceeded the maximum permissible size
    PacketTooLarge = 0x95,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded
    QuotaExceeded = 0x97,
    /// 153[0x99], The Will Payload does not match the specified Payload Format Indicator
    PayloadFormatInvalid = 0x99,
    /// 154[0x9A], The Server does not support retained messages, and Will Retain was set to 1
    RetainNotSupported = 0x9A,
    /// 155[0x9B], The Server does not support the QoS set in Will QoS
    QoSNotSupported = 0x9B,
    /// 156[0x9C], The Client should temporarily use another server
    UseAnotherServer = 0x9C,
    /// 157[0x9D], The Client should permanently use another server
    ServerMoved = 0x9D,
    /// 159[0x9F], The connection rate limit has been exceeded
    ConnectionRateExceeded = 0x9F,
}
