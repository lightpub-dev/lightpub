FROM rabbitmq:3-management

RUN apt-get update -qq && apt-get install -y wget
RUN wget https://github.com/rabbitmq/rabbitmq-delayed-message-exchange/releases/download/v3.13.0/rabbitmq_delayed_message_exchange-3.13.0.ez
RUN cp rabbitmq_delayed_message_exchange-3.13.0.ez /opt/rabbitmq/plugins && rabbitmq-plugins enable rabbitmq_delayed_message_exchange

HEALTHCHECK --interval=30s --timeout=10s --retries=5 --start-period=10s --start-interval=2s CMD rabbitmq-diagnostics -q ping
