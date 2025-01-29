-- CreateEnum
CREATE TYPE "LessonStatus" AS ENUM ('PENDING', 'COMPLETED', 'SKIPPED');

-- CreateEnum
CREATE TYPE "Level" AS ENUM ('A1', 'A2', 'B1', 'B2', 'C1', 'C2');

-- CreateEnum
CREATE TYPE "Role" as ENUM ('USER', 'ADMIN');

-- CreateTable
CREATE TABLE "User" (
    "id" UUID NOT NULL,
    "googleId" BIGINT NOT NULL,
    "displayName" VARCHAR(255) NOT NULL,
    "role" "Role" NOT NULL DEFAULT 'USER',
    "photoUrl" VARCHAR(2048),
    "visible" BOOLEAN NOT NULL DEFAULT TRUE,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "User_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Lesson" (
    "id" UUID NOT NULL,
    "lessonData" JSONB NOT NULL,
    "studiedLang" VARCHAR(2) NOT NULL,
    "lessonLang" VARCHAR(2) NOT NULL,
    "level" "Level" NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Lesson_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "UserLesson" (
    "id" UUID NOT NULL,
    "userId" BIGINT NOT NULL,
    "lessonId" UUID NOT NULL,
    "score" INTEGER NOT NULL DEFAULT 0,
    "status" "LessonStatus" NOT NULL DEFAULT 'PENDING',
    "completedAt" TIMESTAMP(3),
    "nextAvailable" TIMESTAMP(3),

    CONSTRAINT "UserLesson_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "UserStats" (
    "userId" BIGINT NOT NULL,
    "totalScore" INTEGER NOT NULL DEFAULT 0,
    "totalLessons" INTEGER NOT NULL DEFAULT 0,

    CONSTRAINT "UserStats_pkey" PRIMARY KEY ("userId")
);

-- CreateIndex
CREATE UNIQUE INDEX "User_username_key" ON "User"("username");

-- CreateIndex
CREATE INDEX "User_username_idx" ON "User"("username");

-- CreateIndex
CREATE INDEX "User_id_idx" ON "User"("id");

-- CreateIndex
CREATE INDEX "Lesson_studiedLang_lessonLang_level_idx" ON "Lesson"("studiedLang", "lessonLang", "level");

-- CreateIndex
CREATE INDEX "Lesson_id_idx" ON "Lesson"("id");

-- CreateIndex
CREATE INDEX "UserLesson_id_idx" ON "UserLesson"("id");

-- CreateIndex
CREATE INDEX "UserLesson_nextAvailable_idx" ON "UserLesson"("nextAvailable");

-- CreateIndex
CREATE UNIQUE INDEX "UserLesson_userId_lessonId_key" ON "UserLesson"("userId", "lessonId");

-- AddForeignKey
ALTER TABLE "UserLesson" ADD CONSTRAINT "UserLesson_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "UserLesson" ADD CONSTRAINT "UserLesson_lessonId_fkey" FOREIGN KEY ("lessonId") REFERENCES "Lesson"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "UserStats" ADD CONSTRAINT "UserStats_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- CreateFunction
CREATE FUNCTION set_next_available()
    RETURNS TRIGGER AS $$
BEGIN
    IF (NEW.status = 'COMPLETED' AND OLD.status != 'COMPLETED') THEN
        NEW."nextAvailable" := NOW() + INTERVAL '3 days';
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- CreateFunction
CREATE OR REPLACE FUNCTION update_user_stats()
    RETURNS TRIGGER AS $$
BEGIN
    IF (NEW.status = 'COMPLETED' AND OLD.status != 'COMPLETED') THEN
UPDATE "UserStats"
SET
    "totalScore" = "totalScore" + NEW.score,
    "totalLessons" = "totalLessons" + 1
WHERE "userId" = NEW."userId";
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- CreateFunction
CREATE FUNCTION create_user_stats()
    RETURNS TRIGGER AS $$
BEGIN
INSERT INTO "UserStats" ("userId")
VALUES (NEW.id);
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- CreateTrigger
CREATE TRIGGER before_lesson_completion
    BEFORE UPDATE ON "UserLesson"
    FOR EACH ROW
    EXECUTE FUNCTION set_next_available();

-- CreateTrigger
CREATE TRIGGER after_lesson_completion
    AFTER UPDATE ON "UserLesson"
    FOR EACH ROW
    EXECUTE FUNCTION update_user_stats();

-- CreateTrigger
CREATE TRIGGER after_user_insert
    AFTER INSERT ON "User"
    FOR EACH ROW
    EXECUTE FUNCTION create_user_stats();
