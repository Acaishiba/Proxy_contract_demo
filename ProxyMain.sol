// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract CrossChainMessage {
    address public admin;

    enum Status {
        Pending,
        Executed,
        Failed
    }

    struct Message {
        address sender;        // 记录消息的发送者
        address tokenContract; // 目标链代币合约地址
        string tokenName;      // 代币名称
        uint256 amount;        // 转账金额
        string targetAddress;  // 目标链接受地址
        Status status;         // 执行结果
    }

    // 保存所有交互的消息
    Message[] public messages;

    // 事件：记录消息
    event MessageRecorded(
        uint256 indexed messageId,
        address indexed sender,
        address tokenContract,
        string tokenName,
        uint256 amount,
        string targetAddress,
        Status status
    );

    // 事件：执行状态更新
    event StatusUpdated(uint256 indexed messageId, Status status);

    constructor() {
        admin = msg.sender; // 部署合约的账户为管理员
    }

    modifier onlyAdmin() {
        require(msg.sender == admin, "Only admin can update the status.");
        _;
    }

    // 与合约交互，记录一条消息
    function recordMessage(
        address _tokenContract,
        string memory _tokenName,
        uint256 _amount,
        string memory _targetAddress
    ) public {
        Message memory newMessage = Message({
            sender: msg.sender,
            tokenContract: _tokenContract,
            tokenName: _tokenName,
            amount: _amount,
            targetAddress: _targetAddress,
            status: Status.Pending
        });

        messages.push(newMessage);
        emit MessageRecorded(messages.length - 1, msg.sender, _tokenContract, _tokenName, _amount, _targetAddress, Status.Pending);
    }

    // 管理员更新执行结果
    function updateStatus(uint256 _messageId, Status _status) public onlyAdmin {
        require(_messageId < messages.length, "Invalid message ID.");
        messages[_messageId].status = _status;
        emit StatusUpdated(_messageId, _status);
    }

    // 获取消息详情
    function getMessage(uint256 _messageId) public view returns (Message memory) {
        require(_messageId < messages.length, "Invalid message ID.");
        return messages[_messageId];
    }

    // 查询用户发送的所有消息
    function getMessagesBySender(address _sender) public view returns (Message[] memory) {
        uint256 messageCount = 0;

        // 先计算该用户发出的消息数量
        for (uint256 i = 0; i < messages.length; i++) {
            if (messages[i].sender == _sender) {
                messageCount++;
            }
        }

        // 创建一个大小为 messageCount 的数组
        Message[] memory senderMessages = new Message[](messageCount);
        uint256 index = 0;

        // 将该用户的消息存入数组
        for (uint256 i = 0; i < messages.length; i++) {
            if (messages[i].sender == _sender) {
                senderMessages[index] = messages[i];
                index++;
            }
        }

        return senderMessages;
    }

    // 查询所有待执行的消息
    function getPendingMessages() public view returns (Message[] memory) {
        uint256 pendingCount = 0;

        // 先计算状态为待执行的消息数量
        for (uint256 i = 0; i < messages.length; i++) {
            if (messages[i].status == Status.Pending) {
                pendingCount++;
            }
        }

        // 创建一个大小为 pendingCount 的数组
        Message[] memory pendingMessages = new Message[](pendingCount);
        uint256 index = 0;

        // 将状态为待执行的消息存入数组
        for (uint256 i = 0; i < messages.length; i++) {
            if (messages[i].status == Status.Pending) {
                pendingMessages[index] = messages[i];
                index++;
            }
        }

        return pendingMessages;
    }

    // 更换管理员
    function changeAdmin(address _newAdmin) public onlyAdmin {
        require(_newAdmin != address(0), "Invalid address for new admin.");
        admin = _newAdmin;
    }
}
