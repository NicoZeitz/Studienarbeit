from enum import IntFlag

class EntitiesEnum(IntFlag):
    PLAYER_1              = 0b0001
    PLAYER_2              = 0b0010
    BUTTON_INCOME_TRIGGER = 0b0100
    SPECIAL_PATCH         = 0b1000