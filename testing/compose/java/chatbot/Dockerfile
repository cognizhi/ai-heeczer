FROM maven:3.9-eclipse-temurin-17 AS build
WORKDIR /repo
COPY bindings/heeczer-java /repo/bindings/heeczer-java
RUN mvn -f /repo/bindings/heeczer-java/pom.xml -DskipTests package
COPY testing/compose/java/chatbot /repo/testing/compose/java/chatbot
COPY testing/tests/fixtures /fixtures
WORKDIR /repo/testing/compose/java/chatbot
RUN mvn -DskipTests package

FROM eclipse-temurin:17-jre
WORKDIR /app
COPY --from=build /repo/testing/compose/java/chatbot/target/heeczer-test-java-chatbot.jar /app/chatbot.jar
COPY --from=build /fixtures /fixtures
EXPOSE 8000
CMD ["java", "-jar", "/app/chatbot.jar"]
